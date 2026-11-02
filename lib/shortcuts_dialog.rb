# frozen_string_literal: true

require 'adwaita'

require_relative 'i18n'

# ShortcutsDialog lists the keyboard shortcuts.
#
# Ported from upstream's data/resources/ui/shortcuts.ui: an AdwShortcutsDialog
# with a "General" section and a "Bibliography" section, each item naming an
# action rather than an accelerator so that libadwaita shows whatever
# accelerator the application has bound to it. The msgids carry the "shortcut
# window" context upstream gives them.
#
# Upstream never installs the app.shortcuts action the primary menus point at:
# libadwaita installs it when the window has a shortcuts dialog set. The Ruby
# bindings have no equivalent, so the port installs app.shortcuts itself and it
# presents this dialog.
#
class ShortcutsDialog
  include I18n::Translatable

  def sections
    [
      [p_('shortcut window', 'General'), general_items],
      [p_('shortcut window', 'Bibliography'), bibliography_items],
    ]
  end

  def general_items
    [
      [p_('shortcut window', 'Show Shortcuts'), 'app.shortcuts'],
      [p_('shortcut window', 'New Window'), 'app.new-window'],
      [p_('shortcut window', 'Close Window'), 'window.close'],
      [p_('shortcut window', 'Quit'), 'app.quit'],
    ]
  end

  def bibliography_items
    [
      [p_('shortcut window', 'Open'), "win.open('')"],
      [p_('shortcut window', 'Save'), 'win.save'],
      [p_('shortcut window', 'Search'), 'win.search'],
    ]
  end

  def build
    dialog.tap do |widget|
      sections.each do |title, items|
        widget.add(section(title, items))
      end
    end
  end

  def present(parent)
    build.present(parent)
  end

  private

    def dialog = @dialog ||= Adwaita::ShortcutsDialog.new

    def section(title, items)
      Adwaita::ShortcutsSection.new.tap do |widget|
        widget.title = title
        items.each { |label, action| widget.add(item(label, action)) }
      end
    end

    # Positional arguments, and an empty accelerator: the accelerator comes
    # from the action, which is what upstream's action-name items rely on.
    def item(label, action)
      Adwaita::ShortcutsItem.new(label, '').tap do |widget|
        widget.action_name = action
      end
    end
end
