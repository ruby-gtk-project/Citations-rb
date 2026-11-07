# frozen_string_literal: true

require 'json'
require 'fileutils'

# Settings holds the keys of upstream's GSettings schema.
#
# Ported from upstream's data/org.gnome.World.Citations.gschema.xml.in: the
# same eight keys, with the same defaults and the same meanings. The schema
# file itself, with its seven translatable <summary> strings, is ported
# separately to data/org.rubygtkproject.Citations.gschema.xml.in; nothing
# here compiles or installs it.
#
# Upstream reads the keys through gio::Settings, backed by that schema
# compiled into the system's glib-2.0/schemas directory. I did not port that:
# this class reads and writes the same eight keys from a small JSON file
# under XDG_CONFIG_HOME instead. The cost I am naming, not glossing over, is
# that a real GSettings schema gives every key a type the compiler enforces,
# a `gsettings` CLI and dconf-editor entry for free, and change notification
# that works across processes; this JSON file gives none of that — the
# `connect_changed` below only notifies watchers inside this same process.
# I chose it anyway because it needs no install step for a checkout to run,
# which matters more here than those properties do.
#
class Settings
  DEFAULTS = {
    'window-width'            => 800,
    'window-height'           => 600,
    'is-maximized'            => false,
    'recent-files'            => [],
    'custom-book-template'    => '%AUTHOR%, *%TITLE%*, %PUBLISHER%, %CITY%, %YEAR%.',
    'custom-article-template' => '%AUTHOR%, *%TITLE%*, %JOURNAL% **%VOLUME%** (%YEAR%), no. %NUMBER%, %PAGES%.',
    'template-kind'           => 0,
    'format-kind'             => 0,
  }.freeze

  def initialize(path: nil)
    @path = path || default_path
    @values = DEFAULTS.merge(read)
    @watchers = {}
  end

  def location = @path

  def [](key) = @values.fetch(key.to_s, DEFAULTS[key.to_s])

  def []=(key, value)
    key.to_s.then do |name|
      unless @values[name] == value
        @values[name] = value
        write
        notify(name)
      end
    end

    value
  end

  # gio::Settings::connect_changed(Some(key)): the window repopulates its
  # recent files list this way.
  def connect_changed(key, &block)
    @watchers[key.to_s] ||= []
    @watchers[key.to_s] << block
  end

  # Named accessors, so callers do not repeat the string keys.
  DEFAULTS.each_key do |key|
    method_name = key.tr('-', '_')

    define_method(method_name) { self[key] }
    define_method(:"#{method_name}=") { |value| self[key] = value }

    if DEFAULTS[key] == false
      define_method(:"#{method_name}?") { self[key] == true }
    end
  end

  # gio::Settings::strv / set_strv, which the recent files list is stored as.
  def strv(key) = Array(self[key]).map(&:to_s)

  def set_strv(key, values)
    self[key] = Array(values).map(&:to_s)
  end

  # Restores the window geometry recorded by the previous run, as upstream's
  # Window::load_window_size does.
  def apply_to(window)
    window.set_default_size(self['window-width'], self['window-height'])

    if self['is-maximized']
      window.maximize
    end
  end

  # Records the window geometry, as upstream's Window::save_window_size does.
  def remember(window)
    self['window-width'] = window.default_width
    self['window-height'] = window.default_height
    self['is-maximized'] = window.maximized?
  end

  private

    def notify(key)
      @watchers.fetch(key, []).each { |block| block.call(key) }
    end

    def default_path
      File.join(
        ENV.fetch('XDG_CONFIG_HOME', File.expand_path('~/.config')),
        'citations-rb',
        'settings.json',
      )
    end

    def read
      if File.exist?(@path)
        JSON.parse(File.read(@path))
      else
        {}
      end
    rescue JSON::ParserError => e
      warn "Error parsing #{@path}: #{e.message}"
      {}
    end

    def write
      FileUtils.mkdir_p(File.dirname(@path))
      File.write(@path, JSON.pretty_generate(@values))
    end
end
