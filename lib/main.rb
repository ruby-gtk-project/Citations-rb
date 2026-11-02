# frozen_string_literal: true

require 'adwaita'

require_relative 'config'
require_relative 'i18n'
require_relative 'settings'
require_relative 'shortcuts_dialog'
require_relative 'style'
require_relative 'window'

# App is the application: the GApplication, its options, its actions and its
# accelerators.
#
# Ported from upstream's src/application.rs and src/main.rs: the app id and the
# resource base path, the HANDLES_OPEN flag that makes `citations file.bib`
# open that file, the --new-window and --debug options, the app.quit and
# app.new-window actions, the six accelerators, and the gettext bootstrap that
# runs before any widget exists.
#
# Upstream's Application subclasses adw::Application; Adwaita::Application is
# broken in the Ruby bindings, so this is a Gtk::Application with
# Adwaita.init() called at startup, which is what adw::Application does for its
# subclasses.
#
# app.shortcuts is upstream's one implicit action: libadwaita installs it for a
# window that has a shortcuts dialog, which the Ruby bindings cannot do, so it
# is installed here and presents the port's ShortcutsDialog.
#
class App
  include I18n::Translatable

  # upstream's setup_accels
  ACCELERATORS = {
    'app.quit'       => ['<Control>q'],
    'app.new-window' => ['<Control>n'],
    "win.open('')"   => ['<Control>o'],
    'win.save'       => ['<Control>s'],
    'win.search'     => ['<Control>f'],
    'window.close'   => ['<Control>w'],
  }.freeze

  def initialize(app_id: Config::APP_ID, flags: :handles_open, present: true)
    @app_id = app_id
    @flags = flags
    @present = present
    @settings = Settings.new
    @windows = []
    @debug = false
  end

  def build
    app.tap do |application|
      application.signal_connect('startup') { on_startup }
      application.signal_connect('activate') { on_activate }
      application.signal_connect('open') { |_app, files, _hint| open_files(files) }
      application.signal_connect('handle-local-options') do |_app, options|
        handle_local_options(options)
      end

      add_main_options
    end
  end

  # GApplication reads argv[0] as the program name, and the Ruby binding
  # passes the array through untouched, so the name goes back on the front.
  def run(argv = ARGV) = app.run([$PROGRAM_NAME] + argv)

  def windows = @windows

  def app
    @app ||= Gtk::Application.new(@app_id, @flags).tap do |application|
      application.resource_base_path = '/org/gnome/World/Citations/'
    end
  end

  # upstream's open(): one window per file handed to the application.
  def open_files(files)
    files.each do |file|
      new_window.then do |window|
        begin
          window.open(file)
        rescue StandardError => e
          warn "Could not open #{file.uri}: #{e.message}"
        end
      end
    end
  end

  private

    # upstream's add_main_option calls in constructed().
    def add_main_options
      app.add_main_option(
        'new-window',
        'w'.ord,
        GLib::OptionFlags::NONE,
        GLib::OptionArg::NONE,
        _('Open a new window'),
        nil,
      )

      app.add_main_option(
        'debug',
        'd'.ord,
        GLib::OptionFlags::NONE,
        GLib::OptionArg::NONE,
        _('Enable debug messages'),
        nil,
      )
    end

    # upstream's handle_local_options: --new-window on an already running
    # instance activates its new-window action instead of starting a second
    # one. A negative answer means "carry on".
    def handle_local_options(options)
      @debug = options.contains?('debug')
      log_startup_info

      if options.contains?('new-window')
        request_new_window
      else
        -1
      end
    end

    # upstream's run(), which logs the same three lines at info level.
    def log_startup_info
      if @debug
        warn "Citations (#{Config::APP_ID})"
        warn "Version: #{Config::VERSION} (#{Config::PROFILE})"
        warn "Datadir: #{Config::DATADIR}"
      end
    end

    def request_new_window
      app.register

      if app.remote?
        app.activate_action('new-window', nil)

        0
      else
        -1
      end
    end

    # upstream's startup: the style scheme, the icons and the actions.
    def on_startup
      Adwaita.init
      GLib.application_name = _('Citations')
      Style.load
      load_icons
      Gtk::Window.set_default_icon_name(Config::APP_ID)
      setup_gactions
      setup_accels
    end

    # Upstream reads its icons from the gresource bundle; a checkout has them
    # on disk, so the icon theme is pointed at them.
    def load_icons
      Gdk::Display.default.then do |display|
        unless display.nil?
          Gtk::IconTheme.get_for_display(display).add_search_path(Config::ICONDIR)
        end
      end
    end

    def on_activate
      app.active_window.then do |existing|
        if existing.nil?
          new_window
        else
          existing.present
        end
      end
    end

    # upstream's setup_gactions
    def setup_gactions
      app.add_action(quit_action)
      app.add_action(new_window_action)
      app.add_action(shortcuts_action)
    end

    def quit_action
      Gio::SimpleAction.new('quit').tap do |action|
        action.signal_connect('activate') do
          # This is needed to trigger the delete event and saving the window
          # state.
          app.active_window.then do |window|
            unless window.nil?
              window.close
            end
          end

          app.quit
        end
      end
    end

    def new_window_action
      Gio::SimpleAction.new('new-window').tap do |action|
        action.signal_connect('activate') { new_window }
      end
    end

    def shortcuts_action
      Gio::SimpleAction.new('shortcuts').tap do |action|
        action.signal_connect('activate') do
          ShortcutsDialog.new.present(app.active_window)
        end
      end
    end

    def setup_accels
      ACCELERATORS.each do |action, accelerators|
        app.set_accels_for_action(action, accelerators)
      end
    end

    # Each window gets its own window group, as upstream's does.
    def new_window
      Window.new(app, :settings => @settings).tap do |window|
        @windows << window
        window.build
        Gtk::WindowGroup.new.add_window(window.widget)

        if @present
          window.present
        end
      end
    end

end
