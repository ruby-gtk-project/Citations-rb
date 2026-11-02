# frozen_string_literal: true

require 'gtk4'

# Selection is the sidebar's selection model: one selected item at a time, no
# item selected to begin with, and a selection that can be hidden without being
# dropped.
#
# Ported from upstream's src/selection.rs, a GObject implementing both
# gio::ListModel and gtk::SelectionModel so that it can answer "nothing is
# selected" while remembering which item is, which is what lets the collapsed
# navigation view show no highlight and still restore it when it expands.
#
# The Ruby bindings cannot implement a GInterface — `include Gio::ListModel` in
# a GLib::Object subclass leaves the object unusable as a list model, which
# every consumer rejects. So the same behaviour is built on Gtk::SingleSelection
# with `autoselect` off and `can-unselect` on: hiding the selection unselects
# the model while this object keeps the item, and showing it again selects that
# item's current position. The API is upstream's, minus the parts only the
# GInterface needed.
#
class Selection
  INVALID_POSITION = Gtk::INVALID_LIST_POSITION

  def initialize
    @item = nil
    @hide_selection = false
    @on_selection_changed = []
    @updating = false
  end

  # The Gtk::SelectionModel the list view is given.
  def model
    @model ||= Gtk::SingleSelection.new.tap do |selection|
      selection.autoselect = false
      selection.can_unselect = true
      selection.selected = INVALID_POSITION

      selection.signal_connect('notify::selected-item') do
        unless @updating
          @item = selection.selected_item
          notify_selection_changed
        end
      end
    end
  end

  # The list the selection is taken from, upstream's Selection::set_model.
  def list_model=(list_model)
    without_notifying { model.model = list_model }
  end

  def list_model = model.model

  # Called with the selected entry, or nil, as upstream's notify::selected-item
  # handler is connected to.
  def on_selection_changed(&block)
    @on_selection_changed << block
  end

  def selected_item = @item

  def n_items = model.n_items

  # upstream's Selection::selected_item_pos
  def selected_item_pos
    if @item.nil?
      INVALID_POSITION
    else
      find_item_position(@item)
    end
  end

  def set_selected_position(position)
    if position == INVALID_POSITION
      self.selected_item = nil
    else
      model.get_item(position).then { |item| self.selected_item = item }
    end
  end

  def selected_item=(item)
    unless item == @item
      @item = item
      apply_selection
      notify_selection_changed
    end
  end

  def hide_selection? = @hide_selection

  # Hiding the selection drops the highlight without forgetting the entry, so
  # that showing it again puts the highlight back.
  def hide_selection=(hide)
    unless hide == @hide_selection
      @hide_selection = hide
      apply_selection
    end
  end

  private

    def apply_selection
      without_notifying do
        if @hide_selection || @item.nil?
          model.selected = INVALID_POSITION
        else
          find_item_position(@item).then do |position|
            if position == INVALID_POSITION
              model.selected = INVALID_POSITION
            else
              model.selected = position
            end
          end
        end
      end
    end

    def find_item_position(item)
      n_items.times.find { |position| model.get_item(position) == item } ||
        INVALID_POSITION
    end

    def notify_selection_changed
      @on_selection_changed.each { |block| block.call(@item) }
    end

    # The model's own notify::selected-item fires while we are writing to it;
    # those are our own writes and must not be reported as user selections.
    def without_notifying
      @updating = true
      yield
    ensure
      @updating = false
    end
end
