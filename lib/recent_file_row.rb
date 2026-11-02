# frozen_string_literal: true

require 'adwaita'

require_relative 'i18n'
require_relative 'utils'

# RecentFileRow is one row of the "Recent Files" list: the file's name, its
# path, and a button that forgets it.
#
# Ported from the `recent_row` function at the bottom of upstream's
# src/window.rs, which builds one AdwActionRow per recent file, gives it the
# detailed action "win.open::<path>" so that activating the row opens that
# file, and hides the path of files under /run/user, where the document portal
# puts the ones the user picked from another application.
#
class RecentFileRow
  include I18n::Translatable

  PORTAL_PREFIX = '/run/user'

  def initialize(file, settings:)
    @file = file
    @settings = settings
  end

  def build
    row.tap do |action_row|
      path.then do |file_path|
        unless file_path.nil?
          action_row.detailed_action_name = "win.open::#{file_path}"
          action_row.activatable = true
          action_row.add_suffix(clear_button)

          unless file_path.start_with?(PORTAL_PREFIX)
            action_row.subtitle = file_path
          end

          clear_button.signal_connect('clicked') do
            Utils.remove_recent_file(@settings, file_path)
          end
        end
      end

      basename.then do |name|
        unless name.nil?
          action_row.title = name
        end
      end
    end
  end

  private

    def path = @file.path

    def basename
      if path.nil?
        nil
      else
        File.basename(path)
      end
    end

    def row = @row ||= Adwaita::ActionRow.new

    def clear_button
      @clear_button ||= Gtk::Button.new.tap do |button|
        button.icon_name = 'edit-delete-symbolic'
        button.valign = :center
        button.tooltip_text = _('Remove From Recent Files')
        button.add_css_class('flat')
      end
    end
end
