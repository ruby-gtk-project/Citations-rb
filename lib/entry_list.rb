# frozen_string_literal: true

require 'adwaita'

require_relative 'entry'
require_relative 'entry_list_row'
require_relative 'i18n'
require_relative 'selection'

# EntryList is the sidebar: the list of citations in the open bibliography,
# with an empty state for a bibliography that has none.
#
# Ported from upstream's src/entry_list.rs and
# data/resources/ui/entry_list.ui: a stack holding the "No Citations" status
# page and a scrolled GtkListView with the navigation-sidebar and
# padded-sidebar style classes, a SignalListItemFactory building EntryListRows,
# and a FilterListModel whose StringFilter does the substring search.
#
# Upstream's filter runs a gtk::ClosureExpression over the author, title, year
# and citation key. The Ruby bindings have no usable closure expression, so the
# filter watches Entry's "search-text" property, which is those four fields
# joined — same text, same substring match, same ignore-case.
#
class EntryList
  include I18n::Translatable

  def initialize(on_entry_selected:, on_delete:, on_toast:)
    @on_entry_selected = on_entry_selected
    @on_delete = on_delete
    @on_toast = on_toast
    @rows = {}
  end

  def build
    bin.tap do |root|
      root.child = stack

      stack.tap do |pages|
        pages.add_child(empty_list)
        pages.add_child(scrolled_window)

        scrolled_window.child = list_view
      end

      list_view.signal_connect('activate') do |_view, position|
        selection.set_selected_position(position)
      end

      selection.on_selection_changed do |entry|
        @on_entry_selected.call(entry)
      end

      filter_model.filter = filter
      selection.list_model = filter_model
      list_view.model = selection.model
      list_view.factory = factory
    end
  end

  def widget = bin

  # upstream's EntryList::populate
  def populate(bib)
    filter_model.model = bib.model
    set_empty_view(bib.n_items.zero?)

    bib.model.signal_connect('items-changed') do |model, _position, _removed, _added|
      set_empty_view(model.n_items.zero?)
    end
  end

  # upstream's EntryList::filter
  def filter_by(query)
    filter.search = query.to_s
  end

  # upstream's EntryList::set_selected
  def set_selected(position)
    selection.n_items.then do |n_items|
      if position >= n_items && position != Selection::INVALID_POSITION
        warn "Invalid position: #{position}, the list has #{n_items} items"
      else
        selection.hide_selection = false
        selection.set_selected_position(position)
        list_view.scroll_to(position, Gtk::ListScrollFlags::NONE, nil)
      end
    end
  end

  def hide_selection
    selection.hide_selection = true
  end

  def show_selection
    selection.hide_selection = false
  end

  def unselect
    selection.selected_item = nil
  end

  # upstream's EntryList::navigate_next
  def navigate_next
    selection.n_items.then do |n_items|
      position = selection.selected_item_pos

      if position == Selection::INVALID_POSITION && n_items.positive?
        select_and_scroll(0)
      elsif position != Selection::INVALID_POSITION && position + 1 < n_items
        select_and_scroll(position + 1)
      end
    end
  end

  # upstream's EntryList::navigate_previous
  def navigate_previous
    selection.n_items.then do |n_items|
      position = selection.selected_item_pos

      if position == Selection::INVALID_POSITION && n_items.positive?
        select_and_scroll(n_items - 1)
      elsif position != Selection::INVALID_POSITION && position.positive?
        select_and_scroll(position - 1)
      end
    end
  end

  def selection = @selection ||= Selection.new

  def bin = @bin ||= Adwaita::Bin.new
  def stack = @stack ||= Gtk::Stack.new

  def empty_list
    @empty_list ||= Adwaita::StatusPage.new.tap do |page|
      page.icon_name = 'document-text-symbolic'
      # TRANSLATORS Empty state widget
      page.title = _('No Citations')
      page.description = _('Create a new citation')
      page.add_css_class('compact')
    end
  end

  def scrolled_window
    @scrolled_window ||= Gtk::ScrolledWindow.new.tap do |window|
      window.hscrollbar_policy = :never
    end
  end

  def list_view
    @list_view ||= Gtk::ListView.new(nil, nil).tap do |view|
      view.single_click_activate = true
      view.add_css_class('navigation-sidebar')
      view.add_css_class('padded-sidebar')
    end
  end

  def factory
    @factory ||= Gtk::SignalListItemFactory.new.tap do |factory|
      factory.signal_connect('setup') do |_factory, item|
        EntryListRow.new(:on_delete => @on_delete, :on_toast => @on_toast).then do |row|
          item.child = row.build
          @rows[item.child] = row
        end
      end

      factory.signal_connect('bind') do |_factory, item|
        @rows[item.child].entry = item.item
      end
    end
  end

  def filter_model
    @filter_model ||= Gtk::FilterListModel.new(nil, nil)
  end

  def filter
    @filter ||= Gtk::StringFilter.new(
      Gtk::PropertyExpression.new(Entry, nil, 'search-text'),
    ).tap do |string_filter|
      string_filter.match_mode = Gtk::StringFilterMatchMode::SUBSTRING
      string_filter.ignore_case = true
    end
  end

  private

    def select_and_scroll(position)
      selection.set_selected_position(position)
      list_view.scroll_to(position, Gtk::ListScrollFlags::NONE, nil)
    end

    def set_empty_view(is_empty)
      if is_empty
        stack.visible_child = empty_list
      else
        stack.visible_child = scrolled_window
      end
    end

end
