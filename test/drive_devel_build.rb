# frozen_string_literal: true

# Drives the three sites gated on Config::IS_DEVEL with it forced true, so a
# devel build is actually exercised rather than only read: the main window's
# 'devel' CSS class (lib/window.rb, Window#build), the icon both status pages
# show instead of none (lib/window.rb, no_recent_files_status_page and
# recent_files_status_page), and the About dialog showing the raw VERSION
# string rather than the metainfo-derived one (lib/about_dialog.rb, #version).
#
# Config::IS_DEVEL is a frozen module constant set to false for a checkout
# build, so it is removed and redefined here before anything requires the
# widgets that read it, and the real App is driven end to end the same way
# the other drive scripts drive it.

require 'tmpdir'

ENV['XDG_CONFIG_HOME'] = Dir.mktmpdir('citations-config')
ENV['XDG_DATA_HOME'] = Dir.mktmpdir('citations-data')

$LOAD_PATH.unshift(File.expand_path('../lib', __dir__))

require_relative '../lib/config'

Config.send(:remove_const, :IS_DEVEL)
Config.const_set(:IS_DEVEL, true)

require 'main'
require_relative 'gtk_driver'
require_relative '../lib/about_dialog'

app = App.new(:app_id => 'org.rubygtkproject.CitationsDevelDrive', :present => false)

GtkDriver.drive(app, interval: 400) do |d, application|
  d.window { application.windows.first&.widget }

  d.step('assert the devel window carries the devel CSS class') do
    d.check("window has the 'devel' CSS class") do
      application.windows.first.widget.has_css_class?('devel')
    end
  end

  d.step('assert both status pages show the app id as their icon') do
    window = application.windows.first

    d.check('empty-state status page icon is the app id') do
      window.no_recent_files_status_page.icon_name == Config::APP_ID
    end

    d.check('recent-files status page icon is the app id') do
      window.recent_files_status_page.icon_name == Config::APP_ID
    end
  end

  d.step('assert the about dialog shows the raw VERSION string') do
    about = AboutDialog.new(
      :application_icon => Config::APP_ID,
      :version          => Config::VERSION,
    ).build

    d.check('about dialog version equals Config::VERSION, not the metainfo release') do
      about.version == Config::VERSION
    end
  end

  d.step('quit') { application.app.quit }
end
