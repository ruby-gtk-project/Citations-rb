# frozen_string_literal: true

require 'gtk4'

# Style is the application stylesheet.
#
# Ported from upstream's data/resources/style.css, which is loaded from the
# gresource bundle. There is no gresource bundle here, so the same rules are
# carried inline and handed to a Gtk::CssProvider on the default display.
#
module Style
  CSS = <<~CSS
    textview {
      padding: 6px;
    }

    /* NOTE It is not possible to do this with margins on a listview */
    .padded-sidebar {
      padding: 6px;
    }

    scrolledwindow textview.inline,
    scrolledwindow textview.inline text {
      background: none;
    }

    textview.newentrydialog {
      background-color: var(--window-bg-color);
    }

    toolbarview scrolledwindow.card undershoot.top,
    toolbarview scrolledwindow.card undershoot.bottom {
      box-shadow: none;
      background: none;
    }
  CSS

  module_function

  def load
    Gdk::Display.default.then do |display|
      unless display.nil?
        Gtk::StyleContext.add_provider_for_display(
          display,
          provider,
          Gtk::StyleProvider::PRIORITY_APPLICATION,
        )
      end
    end
  end

  def provider
    Gtk::CssProvider.new.tap do |css|
      css.load(:data => CSS)
    end
  end
end
