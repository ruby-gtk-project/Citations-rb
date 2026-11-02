# frozen_string_literal: true

require 'adwaita'

require_relative 'about_dialog'
require_relative 'bib'
require_relative 'config'
require_relative 'entry'
require_relative 'entry_list'
require_relative 'entry_page'
require_relative 'i18n'
require_relative 'new_entry_dialog'
require_relative 'recent_file_row'
require_relative 'settings'
require_relative 'texer'
require_relative 'utils'

# Window is the main window: the recent files screen, the bibliography's
# sidebar and editor, and everything the menus and the keyboard reach.
#
# Ported from upstream's src/window.rs and data/resources/ui/window.ui: an
# AdwApplicationWindow holding a toast overlay over a stack whose two children
# are the recent files screen and the navigation split view, a breakpoint that
# collapses the split view under 600sp, the eleven win.* actions, the drop
# target that opens a dropped .bib, the unsaved-changes dialogs, and the window
# state saved to settings on close.
#
class Window
  include I18n::Translatable

  # The bibliography actions that only make sense once a bibliography is open.
  ENTRY_ACTIONS = %w[new_entry save delete_entry].freeze

  # The undo information a deleted citation leaves behind, upstream's UndoInfo.
  UndoInfo = Struct.new(:entry, :position, :toast)

  def initialize(application, settings: nil)
    @application = application
    @settings = settings || Settings.new
    @bib = nil
    @file = nil
    @undo_entry = nil
    @force_close = false
  end

  attr_reader :settings

  def build
    window.tap do |win|
      win.content = toast_overlay
      win.add_breakpoint(breakpoint)

      toast_overlay.child = main_stack

      main_stack.tap do |stack|
        stack.add_child(recent_files_stack)
        stack.add_child(nav_split_view)

        recent_files_stack.tap do |pages|
          pages.add_child(empty_state_stack_page)
          pages.add_child(recent_files_stack_page)

          empty_state_stack_page.tap do |page|
            page.add_top_bar(empty_header_bar)
            page.content = no_recent_files_status_page

            empty_header_bar.pack_end(empty_menu_button)
            no_recent_files_status_page.child = open_button
          end

          recent_files_stack_page.tap do |page|
            page.add_top_bar(recent_header_bar)
            page.content = recent_files_status_page

            recent_header_bar.pack_end(recent_menu_button)
            recent_files_status_page.child = recent_files_clamp
            recent_files_clamp.child = recent_files_group

            recent_files_group.tap do |group|
              group.header_suffix = recent_open_button
              group.add(recent_files_list)

              recent_open_button.child = recent_open_button_content
            end
          end
        end

        nav_split_view.tap do |view|
          view.sidebar = sidebar_page
          view.content = content_page

          sidebar_toolbar.tap do |toolbar|
            toolbar.add_top_bar(sidebar_header_bar)
            toolbar.add_top_bar(search_bar)
            toolbar.content = entry_list.build

            sidebar_header_bar.tap do |bar|
              bar.pack_start(new_entry_button)
              bar.pack_end(appmenu_button)
              bar.pack_end(search_button)
            end

            search_bar.child = search_entry
          end

          content_toolbar.tap do |toolbar|
            toolbar.add_top_bar(content_header_bar)
            toolbar.content = entry_page.build

            content_header_bar.pack_end(secondary_menu_button)
          end
        end
      end

      install_actions
      connect_signals

      win.add_controller(drop_target)
      @settings.apply_to(win)
      @settings.connect_changed('recent-files') { populate_recent_files }
      populate_recent_files
    end
  end

  def widget = window

  def present
    window.present
  end

  def bib = @bib

  # upstream's Window::send_toast
  def send_toast(text)
    toast_overlay.add_toast(Adwaita::Toast.new(text))
  end

  # upstream's Window::open
  def open(file)
    read_contents(file).then do |data|
      Bib.new.then do |bib|
        bib.parse(data)

        unless bib.n_items.zero?
          entry_page.set_empty_status_page
        end

        entry_list.populate(bib)
        @bib = bib
        @file = file
        search_bar.key_capture_widget = window
        main_stack.visible_child = nav_split_view

        action('new_biblio').enabled = false
        ENTRY_ACTIONS.each { |name| action(name).enabled = true }

        remember_recent_file(file)
      end
    end
  end

  # upstream's Window::save
  def save
    unless @file.nil? || @bib.nil?
      @bib.save_to_file(@file)
      send_toast(_('Bibliography saved'))
    end
  end

  # upstream's Window::delete_entry
  def delete_entry(entry)
    @bib.find_pos(entry).then do |position|
      unless position.nil?
        delete_at(entry, position)
      end
    end
  end

  def window
    @window ||= Adwaita::ApplicationWindow.new(@application).tap do |win|
      win.title = _('Citations')
    end
  end

  # upstream's AdwBreakpoint: under 600sp the split view collapses.
  def breakpoint
    @breakpoint ||= Adwaita::Breakpoint.new(
      Adwaita::BreakpointCondition.parse('max-width: 600sp'),
    ).tap do |bp|
      bp.add_setter(nav_split_view, 'collapsed', true)
    end
  end

  def toast_overlay = @toast_overlay ||= Adwaita::ToastOverlay.new
  def main_stack = @main_stack ||= Gtk::Stack.new
  def recent_files_stack = @recent_files_stack ||= Gtk::Stack.new

  def empty_state_stack_page
    @empty_state_stack_page ||= Adwaita::ToolbarView.new
  end

  def recent_files_stack_page
    @recent_files_stack_page ||= Adwaita::ToolbarView.new
  end

  def empty_header_bar = @empty_header_bar ||= Adwaita::HeaderBar.new
  def recent_header_bar = @recent_header_bar ||= Adwaita::HeaderBar.new

  def empty_menu_button = @empty_menu_button ||= primary_menu_button(primary_menu_empty)
  def recent_menu_button = @recent_menu_button ||= primary_menu_button(primary_menu_empty)

  def primary_menu_button(model)
    Gtk::MenuButton.new.tap do |button|
      button.icon_name = 'open-menu-symbolic'
      button.menu_model = model
      button.tooltip_text = _('Main Menu')
      button.primary = true
    end
  end

  # upstream's no_recent_files_status_page
  def no_recent_files_status_page
    @no_recent_files_status_page ||= Adwaita::StatusPage.new.tap do |page|
      page.vexpand = true
      page.icon_name = Config::APP_ID
      # TRANSLATORS BibTeX is a proper name
      page.title = _('Open a BibTeX File')
      page.description = _('Or drag-and-drop a file')
      page.add_css_class('icon-dropshadow')
    end
  end

  def open_button
    @open_button ||= Gtk::Button.new.tap do |button|
      button.label = _('_Open')
      button.use_underline = true
      button.halign = :center
      button.action_name = 'win.open'
      button.action_target = GLib::Variant.new('')
      button.can_shrink = true
      button.add_css_class('pill')
      button.add_css_class('suggested-action')
    end
  end

  # upstream's recent_files_status_page
  def recent_files_status_page
    @recent_files_status_page ||= Adwaita::StatusPage.new.tap do |page|
      page.title = _('Manage your Citations')
      page.icon_name = Config::APP_ID
      page.add_css_class('icon-dropshadow')
    end
  end

  def recent_files_clamp
    @recent_files_clamp ||= Adwaita::Clamp.new.tap do |clamp|
      clamp.maximum_size = 450
    end
  end

  def recent_files_group
    @recent_files_group ||= Adwaita::PreferencesGroup.new.tap do |group|
      group.title = _('Recent Files')
    end
  end

  def recent_open_button
    @recent_open_button ||= Gtk::Button.new.tap do |button|
      button.action_name = 'win.open'
      button.action_target = GLib::Variant.new('')
      button.add_css_class('flat')
    end
  end

  def recent_open_button_content
    @recent_open_button_content ||= Adwaita::ButtonContent.new.tap do |content|
      content.icon_name = 'document-open-symbolic'
      content.label = _('_Open')
      content.use_underline = true
      content.can_shrink = true
    end
  end

  def recent_files_list
    @recent_files_list ||= Gtk::ListBox.new.tap do |list|
      list.selection_mode = :none
      list.add_css_class('boxed-list')
    end
  end

  def nav_split_view
    @nav_split_view ||= Adwaita::NavigationSplitView.new.tap do |view|
      view.min_sidebar_width = 300
      view.max_sidebar_width = 360
      view.sidebar_width_fraction = 0.35
      view.show_content = false
    end
  end

  # NavigationPage takes its child positionally, so the toolbars exist first.
  def sidebar_page
    @sidebar_page ||= Adwaita::NavigationPage.new(sidebar_toolbar, _('Sidebar'))
  end

  def content_page
    @content_page ||= Adwaita::NavigationPage.new(content_toolbar, _('Content'))
  end

  def sidebar_toolbar = @sidebar_toolbar ||= Adwaita::ToolbarView.new
  def content_toolbar = @content_toolbar ||= Adwaita::ToolbarView.new

  def sidebar_header_bar
    @sidebar_header_bar ||= Adwaita::HeaderBar.new.tap do |bar|
      bar.show_title = false
    end
  end

  def content_header_bar
    @content_header_bar ||= Adwaita::HeaderBar.new.tap do |bar|
      bar.show_title = false
    end
  end

  def new_entry_button
    @new_entry_button ||= Gtk::Button.new.tap do |button|
      button.sensitive = false
      button.icon_name = 'list-add-symbolic'
      button.action_name = 'win.new_entry'
      button.tooltip_text = _('Add New Citation')
    end
  end

  def appmenu_button = @appmenu_button ||= primary_menu_button(primary_menu)

  def search_button
    @search_button ||= Gtk::ToggleButton.new.tap do |button|
      button.icon_name = 'system-search-symbolic'
      button.tooltip_text = _('Search')
    end
  end

  def secondary_menu_button
    @secondary_menu_button ||= Gtk::MenuButton.new.tap do |button|
      button.icon_name = 'view-more-symbolic'
      button.tooltip_text = _('Secondary Menu')
      button.menu_model = entry_menu
      button.visible = false
    end
  end

  def search_bar = @search_bar ||= Gtk::SearchBar.new

  def search_entry
    @search_entry ||= Gtk::SearchEntry.new.tap do |entry|
      entry.placeholder_text = _('Search…')
    end
  end

  def entry_list
    @entry_list ||= EntryList.new(
      :on_entry_selected => method(:on_entry_selected),
      :on_delete         => method(:delete_entry),
      :on_toast          => method(:send_toast),
    )
  end

  def entry_page
    @entry_page ||= EntryPage.new(
      :settings   => @settings,
      :on_toast   => method(:send_toast),
      :key_exists => ->(key) { !@bib.nil? && @bib.key_exists?(key) },
    )
  end

  # upstream's primary_menu_empty
  def primary_menu_empty
    @primary_menu_empty ||= Gio::Menu.new.tap do |menu|
      menu.append_section(nil, empty_file_section)
      menu.append_section(nil, about_section)
    end
  end

  def empty_file_section
    Gio::Menu.new.tap do |section|
      section.append(_('_Create New Bibliography'), 'win.new_biblio')
      section.append(_('_Open'), "win.open('')")
    end
  end

  # upstream's primary_menu
  def primary_menu
    @primary_menu ||= Gio::Menu.new.tap do |menu|
      menu.append_section(nil, file_section)
      menu.append_section(nil, about_section)
    end
  end

  def file_section
    Gio::Menu.new.tap do |section|
      section.append(_('_Save'), 'win.save')
      section.append(_('_Open'), "win.open('')")
      section.append(_('New _Window'), 'app.new-window')
    end
  end

  def about_section
    Gio::Menu.new.tap do |section|
      section.append(_('_Keyboard Shortcuts'), 'app.shortcuts')
      # TRANSLATORS About dialog
      section.append(_('_About Citations'), 'win.about')
    end
  end

  # upstream's entry_menu
  def entry_menu
    @entry_menu ||= Gio::Menu.new.tap do |menu|
      menu.append_section(nil, search_section)
      menu.append_section(nil, copy_section)
      menu.append_section(nil, entry_delete_section)
    end
  end

  def search_section
    Gio::Menu.new.tap do |section|
      section.append(_('_Search in Google Scholar'), 'win.search-google-scholar')
      # TRANSLATORS arXiv is a proper name
      section.append(_('_Search in arXiv'), 'win.search-arxiv')
    end
  end

  def copy_section
    Gio::Menu.new.tap do |section|
      section.append(_('_Copy BibTeX Entry'), 'win.copy_entry_bibtex')
    end
  end

  def entry_delete_section
    Gio::Menu.new.tap do |section|
      section.append(_('_Delete'), 'win.delete_entry')
    end
  end

  def bib_file_dialog
    @bib_file_dialog ||= Gtk::FileDialog.new.tap do |chooser|
      chooser.filters = bib_filters
    end
  end

  # upstream's new_biblio_action dialog
  def new_biblio_dialog
    @new_biblio_dialog ||= Gtk::FileDialog.new.tap do |chooser|
      chooser.filters = bib_filters
      chooser.title = _('Create New Bibliography')
      chooser.accept_label = _('_Create')
      # TRANSLATORS This a file name, .bib will be appended e.g.
      # bibliography.bib, please only use alphabetic characters.
      chooser.initial_name = "#{_('bibliography')}.bib"
    end
  end

  def bib_filters
    Gio::ListStore.new(Gtk::FileFilter.gtype).tap do |filters|
      filters.append(bib_filter)
    end
  end

  def bib_filter
    Gtk::FileFilter.new.tap do |filter|
      filter.name = _('BibTeX Files')
      filter.add_mime_type('text/x-bibtex')
      filter.add_pattern('*.bib')
    end
  end

  # upstream's file-drop-target: the first dropped file is opened.
  def drop_target
    @drop_target ||= Gtk::DropTarget.new(Gdk::FileList.gtype, [:copy, :move]).tap do |target|
      target.name = 'file-drop-target'

      target.signal_connect('drop') do |_target, value, _x, _y|
        dropped_file(value).then do |file|
          if file.nil?
            false
          else
            drag_handler(file)

            true
          end
        end
      end
    end
  end

  def dropped_file(value)
    if value.respond_to?(:files)
      value.files.first
    else
      nil
    end
  end

  # upstream's drag_handler
  def drag_handler(file)
    if !@bib.nil? && @bib.modified?
      ask_opening_new_bib_with_changes { open(file) }
    else
      open(file)
    end
  end

  # upstream's new_entry
  def new_entry(data)
    entry = Entry.from_bibtex(data)

    if @bib.key_exists?(entry.citation_key)
      send_toast(_('An entry with the given citation key already exists'))
    else
      search_bar.search_mode = false
      entry_list.set_selected(@bib.add_entry(entry))
    end
  rescue StandardError => e
    send_toast(_('Could not add entry'))
    warn "Could not get valid entry from dialog: #{e.message}"
  end

  private

    def delete_at(entry, position)
      deletion_toast.tap do |toast|
        @undo_entry.then do |previous|
          @undo_entry = UndoInfo.new(entry, position, toast)

          unless previous.nil?
            previous.toast.dismiss
          end
        end

        entry_list.unselect
        @bib.remove_pos(position)

        toast_overlay.add_toast(toast)
        nav_split_view.show_content = false

        if @bib.n_items.zero?
          entry_page.set_no_items_placeholder_view
        end
      end
    end

    def deletion_toast
      Adwaita::Toast.new(_('Citation Deleted')).tap do |toast|
        toast.action_name = 'win.undo_delete'
        toast.button_label = _('Undo')
        toast.signal_connect('dismissed') { @undo_entry = nil }
      end
    end

    def read_contents(file)
      file.load_contents(nil).then do |result|
        bytes = result[1]

        if bytes.is_a?(Array)
          bytes.pack('C*').force_encoding(Encoding::UTF_8)
        else
          bytes.to_s.dup.force_encoding(Encoding::UTF_8)
        end
      end
    end

    def remember_recent_file(file)
      file.path.then do |path|
        unless path.nil?
          @settings.strv('recent-files').then do |recent_files|
            unless recent_files.include?(path)
              @settings.set_strv('recent-files', recent_files + [path])
            end
          end
        end
      end
    end

    # upstream's Window::populate_recent_files
    def populate_recent_files
      recent_files.then do |files|
        Gio::ListStore.new(Gio::File.gtype).tap do |model|
          files.each { |file| model.append(file) }
          recent_files_list.bind_model(model) do |file|
            RecentFileRow.new(file, :settings => @settings).build
          end
        end

        if files.empty?
          recent_files_stack.visible_child = empty_state_stack_page
        else
          recent_files_stack.visible_child = recent_files_stack_page
        end
      end
    end

    def recent_files
      @settings.strv('recent-files')
        .reverse
        .map { |path| Gio::File.new_for_path(path) }
        .select { |file| File.exist?(file.path) }
    end

    # The win.* actions, upstream's install_action calls.
    def install_actions
      window.add_action(open_action)
      window.add_action(simple_action('new_biblio') { new_biblio_action })
      window.add_action(simple_action('new_entry') { new_entry_dialog })
      window.add_action(simple_action('save') { save })
      window.add_action(simple_action('delete_entry') { delete_selected_entry })
      window.add_action(simple_action('copy_entry_bibtex') { copy_selected_entry_bibtex })
      window.add_action(simple_action('search-google-scholar') { search_google_scholar_entry })
      window.add_action(simple_action('search-arxiv') { search_arxiv_entry })
      window.add_action(simple_action('undo_delete') { undo_delete })
      window.add_action(simple_action('about') { show_about_dialog })
      window.add_action(simple_action('search') { start_search_action })

      ENTRY_ACTIONS.each { |name| action(name).enabled = false }
    end

    def simple_action(name, &block)
      Gio::SimpleAction.new(name).tap do |action|
        action.signal_connect('activate') { block.call }
      end
    end

    # win.open carries the path to open, or an empty string to ask for one.
    def open_action
      Gio::SimpleAction.new('open', GLib::VariantType.new('s')).tap do |action|
        action.signal_connect('activate') do |_action, parameter|
          open_action_for(parameter.get_string)
        end
      end
    end

    def action(name) = window.lookup_action(name)

    def connect_signals
      window.signal_connect('close-request') { on_close_request }

      nav_split_view.signal_connect('notify::collapsed') { on_collapsed_notify }
      content_page.signal_connect('hidden') { entry_list.hide_selection }

      search_bar.signal_connect('notify::search-mode-enabled') do
        unless search_bar.search_mode?
          search_entry.text = ''
        end
      end

      search_entry.signal_connect('changed') { entry_list.filter_by(search_entry.text) }
      search_entry.signal_connect('previous-match') { entry_list.navigate_previous }
      search_entry.signal_connect('next-match') { entry_list.navigate_next }

      search_button.bind_property(
        'active',
        search_bar,
        'search-mode-enabled',
        :bidirectional,
      )
    end

    # upstream's on_entry_selected
    def on_entry_selected(entry)
      entry_page.entry = entry
      secondary_menu_button.visible = !entry.nil?
      nav_split_view.show_content = true
    end

    # upstream's on_collapsed_notify
    def on_collapsed_notify
      if nav_split_view.collapsed?
        entry_list.hide_selection
      else
        entry_list.show_selection
        nav_split_view.show_content = true
      end
    end

    # upstream's close_request: the window state is written, and a modified
    # bibliography asks before the window goes away.
    def on_close_request
      @settings.remember(window)

      if !@bib.nil? && @bib.modified? && !@force_close
        show_save_dialog

        true
      else
        false
      end
    end

    # upstream's open_action
    def open_action_for(path)
      if !@bib.nil? && @bib.modified?
        ask_opening_new_bib_with_changes { open_action_inner(path) }
      else
        open_action_inner(path)
      end
    end

    # upstream's open_action_inner
    def open_action_inner(path)
      if path.to_s.empty?
        bib_file_dialog.open(window) { |chooser, result| open_chosen(chooser, result) }
      else
        open(Gio::File.new_for_path(path))
      end
    end

    def open_chosen(chooser, result)
      chooser.open_finish(result).then do |file|
        unless file.nil?
          open(file)
        end
      end
    rescue StandardError => e
      warn "Could not open file: #{e.message}"
    end

    # upstream's new_biblio_action
    def new_biblio_action
      new_biblio_dialog.save(window) { |chooser, result| create_chosen(chooser, result) }
    end

    def create_chosen(chooser, result)
      chooser.save_finish(result).then do |file|
        unless file.nil?
          File.write(file.path, '')
          open(file)
        end
      end
    rescue StandardError => e
      warn "Could not create file: #{e.message}"
    end

    # upstream's new_entry_dialog
    def new_entry_dialog
      NewEntryDialog.new.run(window) do |data|
        unless data.nil?
          new_entry(data)
        end
      end
    end

    def delete_selected_entry
      entry_page.entry.then do |entry|
        unless entry.nil?
          delete_entry(entry)
        end
      end
    end

    # upstream's copy_entry_bibtex
    def copy_selected_entry_bibtex
      entry_page.entry.then do |entry|
        unless entry.nil?
          window.display.clipboard.set(entry.serialize)
          send_toast(_('Copied'))
        end
      end
    end

    def undo_delete
      @undo_entry.then do |undo|
        unless undo.nil?
          @undo_entry = nil
          @bib.add_entry_at_pos(undo.entry, undo.position)
        end
      end
    end

    # upstream's search_google_schoolar_entry
    def search_google_scholar_entry
      selected_entry.then do |entry|
        unless entry.nil?
          launch_uri(
            Utils.build_google_scholar_url(
              Texer.texer(entry.author),
              Texer.texer(entry.title),
            ),
          )
        end
      end
    end

    # upstream's search_arxiv_entry
    def search_arxiv_entry
      selected_entry.then do |entry|
        unless entry.nil?
          launch_uri(
            Utils.build_arxiv_url(
              Texer.texer(entry.author),
              Texer.texer(entry.title),
            ),
          )
        end
      end
    end

    def selected_entry = entry_page.entry

    def launch_uri(uri)
      Gtk::UriLauncher.new(uri).launch(window)
    rescue StandardError => e
      warn "Failed to launch uri #{uri}: #{e.message}"
    end

    # upstream's start_search_action
    def start_search_action
      if nav_split_view.collapsed?
        entry_list.hide_selection
        nav_split_view.show_content = false
      end

      search_bar.search_mode = !search_bar.search_mode?
    end

    def show_about_dialog
      AboutDialog.new(
        :application_icon => Config::APP_ID,
        :version          => Config::VERSION,
      ).present(window)
    end

    # upstream's ask_opening_new_bib_with_changes
    def ask_opening_new_bib_with_changes(&on_proceed)
      unsaved_changes_dialog(
        # NOTE Dialog subtitle which informs the user about unsaved changes
        # more detailed.
        _('Do you want to write all changes to the bibliography before opening the new one?'),
        _('_Open Without Saving'),
        _("_Don't Open"),
        _('_Save and Open'),
      ).tap do |dialog|
        dialog.signal_connect('response') do |_dialog, response|
          on_open_response(response, &on_proceed)
        end

        dialog.present(window)
      end
    end

    def on_open_response(response, &on_proceed)
      case response
      when 'discard' then on_proceed.call
      when 'save' then save_then(&on_proceed)
      end
    end

    def save_then(&on_proceed)
      save
      on_proceed.call
    rescue StandardError => e
      warn "Could not save bib file: #{e.message}"
    end

    # upstream's show_save_dialog
    def show_save_dialog
      unsaved_changes_dialog(
        # NOTE Dialog subtitle which informs the user about unsaved changes
        # more detailed.
        _('Do you want to write all changes to the bibliography?'),
        _('_Quit Without Saving'),
        _("_Don't Quit"),
        _('_Save and Quit'),
      ).tap do |dialog|
        dialog.signal_connect('response') do |_dialog, response|
          on_quit_response(response)
        end

        dialog.present(window)
      end
    end

    def on_quit_response(response)
      case response
      when 'discard' then force_close
      when 'save' then save_and_close
      end
    end

    def save_and_close
      save
      window.close
    rescue StandardError => e
      warn "Could not save bib file: #{e.message}"
    end

    def force_close
      @force_close = true
      window.close
    end

    def unsaved_changes_dialog(body, discard, cancel, confirm)
      # NOTE Dialog title which informs the user about unsaved changes.
      Adwaita::AlertDialog.new(_('Unsaved Changes'), body).tap do |dialog|
        dialog.add_response('discard', discard)
        dialog.add_response('cancel', cancel)
        dialog.add_response('save', confirm)
        dialog.set_response_appearance('discard', Adwaita::ResponseAppearance::DESTRUCTIVE)
        dialog.default_response = 'save'
      end
    end

end
