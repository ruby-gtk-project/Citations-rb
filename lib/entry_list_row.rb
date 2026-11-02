# frozen_string_literal: true

require 'adwaita'

require_relative 'i18n'
require_relative 'texer'
require_relative 'utils'

# EntryListRow is one row of the sidebar: the author, the year, the title, and
# a context menu.
#
# Ported from upstream's src/entry_list_row.rs and
# data/resources/ui/entry_list_row.ui, a gtk::Widget subclass with a BinLayout
# whose box holds the author and year on one line and the title under it.
#
# Upstream keeps the labels in step with a glib::BindingGroup. The Ruby
# bindings have no BindingGroup, so `entry=` connects to the entry's notify
# signal itself and disconnects from the previous entry — list rows are
# recycled, so the disconnect matters.
#
class EntryListRow
  include I18n::Translatable

  attr_reader :entry

  def initialize(on_delete:, on_toast:)
    @on_delete = on_delete
    @on_toast = on_toast
    @entry = nil
    @handler = nil
  end

  def build
    child.tap do |box|
      box.append(top_box)
      box.append(title_label)

      top_box.tap do |top|
        top.append(author_label)
        top.append(year_label)
      end

      popover.set_parent(box)

      box.insert_action_group('entry-row', actions)
      box.add_controller(menu_gesture)
      box.add_controller(menu_long_gesture)
      box.add_controller(shortcuts)

      menu_gesture.signal_connect('pressed') do |gesture, _n_press, x, y|
        menu_gesture_pressed(gesture, x, y)
      end

      menu_long_gesture.signal_connect('pressed') do |gesture, x, y|
        open_context_menu(x, y)
        gesture.state = Gtk::EventSequenceState::CLAIMED
      end
    end
  end

  def entry=(entry)
    unless entry == @entry
      disconnect_entry
      @entry = entry
      connect_entry
      update_labels
    end
  end

  # upstream's open_context_menu: a right click or a long press points the
  # popover at the pointer, the Menu key points it at the row's corner.
  def open_context_menu(x, y)
    actions.lookup_action('open-pdf').enabled = Utils.has_pdf?(@entry.citation_key)
    popover.pointing_to = pointing_rectangle(x, y)
    popover.popup
  end

  def widget = child

  # The row's own menu, upstream's row_menu.
  def row_menu
    @row_menu ||= Gio::Menu.new.tap do |menu|
      menu.append_section(nil, source_section)
      menu.append_section(nil, delete_section)
    end
  end

  def child
    @child ||= Gtk::Box.new(:vertical, 6).tap do |box|
      box.margin_start = 6
      box.margin_end = 6
      box.margin_top = 12
      box.margin_bottom = 12
    end
  end

  def top_box = @top_box ||= Gtk::Box.new(:horizontal, 6)

  def author_label
    @author_label ||= Gtk::Label.new.tap do |label|
      label.hexpand = true
      label.xalign = 0
      label.ellipsize = :end
      label.lines = 2
      label.wrap = true
      label.wrap_mode = :word_char
      label.add_css_class('title')
    end
  end

  def year_label
    @year_label ||= Gtk::Label.new.tap do |label|
      label.yalign = 0
      label.add_css_class('dimmed')
      label.add_css_class('numeric')
    end
  end

  def title_label
    @title_label ||= Gtk::Label.new.tap do |label|
      label.ellipsize = :end
      label.lines = 2
      label.wrap = true
      label.wrap_mode = :word_char
      label.xalign = 0
      label.hexpand = true
      label.add_css_class('subtitle')
    end
  end

  def popover
    @popover ||= Gtk::PopoverMenu.new(row_menu).tap do |menu|
      menu.has_arrow = false
      menu.halign = :start
      menu.position = :bottom
    end
  end

  def menu_gesture
    @menu_gesture ||= Gtk::GestureClick.new.tap do |gesture|
      gesture.button = 0
      gesture.exclusive = true
    end
  end

  def menu_long_gesture
    @menu_long_gesture ||= Gtk::GestureLongPress.new.tap do |gesture|
      gesture.delay_factor = 2
      gesture.touch_only = true
      gesture.exclusive = true
    end
  end

  # Menu and <Shift>F10 open the context menu, as upstream's binding actions
  # do.
  def shortcuts
    @shortcuts ||= Gtk::ShortcutController.new.tap do |controller|
      controller.add_shortcut(popup_shortcut(Gdk::Keyval::KEY_Menu, 0))
      controller.add_shortcut(
        popup_shortcut(Gdk::Keyval::KEY_F10, Gdk::ModifierType::SHIFT_MASK),
      )
    end
  end

  def popup_shortcut(keyval, modifiers)
    Gtk::Shortcut.new(
      Gtk::KeyvalTrigger.new(keyval, modifiers),
      Gtk::NamedAction.new('entry-row.popup'),
    )
  end

  private

    def source_section
      Gio::Menu.new.tap do |section|
        section.append(_('_Open PDF'), 'entry-row.open-pdf')
        section.append(_('_Copy Key'), 'entry-row.copy-key')
      end
    end

    def delete_section
      Gio::Menu.new.tap do |section|
        section.append(_('_Delete'), 'entry-row.delete')
      end
    end

    def actions
      @actions ||= Gio::SimpleActionGroup.new.tap do |group|
        group.add_action(open_pdf_action)
        group.add_action(copy_key_action)
        group.add_action(delete_action)
        group.add_action(popup_action)
      end
    end

    def open_pdf_action
      Gio::SimpleAction.new('open-pdf').tap do |action|
        action.enabled = false
        action.signal_connect('activate') do
          begin
            Utils.open_pdf(@entry.citation_key, child.root)
          rescue StandardError => e
            warn "Could not open pdf: #{e.message}"
          end
        end
      end
    end

    def copy_key_action
      Gio::SimpleAction.new('copy-key').tap do |action|
        action.signal_connect('activate') do
          clipboard.set(@entry.citation_key)
          @on_toast.call(_('Copied'))
        end
      end
    end

    def delete_action
      Gio::SimpleAction.new('delete').tap do |action|
        action.signal_connect('activate') do
          @on_delete.call(@entry)
        end
      end
    end

    def popup_action
      Gio::SimpleAction.new('popup').tap do |action|
        action.signal_connect('activate') do
          open_context_menu(-1.0, -1.0)
        end
      end
    end

    def clipboard = child.display.clipboard

    # Only a context-menu click opens the menu; anything else is left to the
    # list view, which is what makes a plain click still select the row.
    def menu_gesture_pressed(gesture, x, y)
      gesture.current_event.then do |event|
        if event.nil? || !event.triggers_context_menu?
          gesture.state = Gtk::EventSequenceState::DENIED
        else
          open_context_menu(x, y)
          gesture.state = Gtk::EventSequenceState::CLAIMED
          gesture.reset
        end
      end
    end

    def pointing_rectangle(x, y)
      if x > -0.5 && y > -0.5
        Gdk::Rectangle.new(
          x.to_i,
          y.to_i,
          0,
          0,
        )
      elsif child.direction == Gtk::TextDirection::RTL
        Gdk::Rectangle.new(
          child.width,
          child.height,
          0,
          0,
        )
      else
        Gdk::Rectangle.new(
          0,
          child.height,
          0,
          0,
        )
      end
    end

    def connect_entry
      unless @entry.nil?
        @handler = @entry.signal_connect('notify') { update_labels }
      end
    end

    def disconnect_entry
      unless @entry.nil? || @handler.nil?
        @entry.signal_handler_disconnect(@handler)
        @handler = nil
      end
    end

    def update_labels
      author_label.text = pretty_author
      title_label.text = pretty_title
      year_label.text = year
    end

    def year
      if @entry.nil?
        ''
      else
        @entry.year
      end
    end

    def pretty_author
      if @entry.nil?
        ''
      else
        Texer.texer(@entry.author).then do |author|
          if author.empty?
            _('Author not set')
          else
            author
          end
        end
      end
    end

    def pretty_title
      if @entry.nil?
        ''
      else
        Texer.texer(@entry.title).then do |title|
          if title.empty?
            _('Title not set')
          else
            title
          end
        end
      end
    end

end
