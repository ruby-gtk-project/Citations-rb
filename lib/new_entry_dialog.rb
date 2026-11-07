# frozen_string_literal: true

require 'adwaita'
require 'gtksourceview5'

require_relative 'entry_type'
require_relative 'i18n'
require_relative 'utils'

# NewEntryDialog asks for a new citation: a key and a type, a BibTeX snippet,
# or a DOI to look up.
#
# Ported from upstream's src/new_entry_dialog.rs and
# data/resources/ui/new_entry_dialog.ui: an AdwDialog titled "Add new Entry"
# holding an AdwViewStack with the "New", "From BibTeX" and "From DOI" pages, a
# view switcher bar at the bottom, and a Create button that stays insensitive
# until the visible page has something to create from.
#
# Upstream hands its answer back over a oneshot channel it awaits; here `run`
# takes the block to call with the BibTeX text, or with nil when the dialog was
# dismissed, which is the same contract without the executor.
#
# The snippet editor is a real GtkSource::View with the bibtex language and
# the Adwaita/Adwaita-dark style scheme, switched with the dark preference,
# matching upstream's constructed and update_style_scheme.
#
class NewEntryDialog
  include I18n::Translatable

  SAMPLE_ENTRY = <<~BIBTEX.chomp
    @book{lamport1985i1,
      title={\\LaTeX: A Document Preparation System},
      author={Lamport, Leslie},
      year={1991},
      publisher={Addison-Wesley}
    }
  BIBTEX

  # upstream's AdwComboRow starts on Article rather than Other.
  DEFAULT_TYPE_INDEX = 1

  def initialize
    @answered = false
    @text_removed = false
  end

  # upstream's NewEntryDialog::run.
  def run(window, &block)
    @on_answer = block

    build.present(window)
  end

  def build
    dialog.tap do |root|
      root.child = toolbar_view

      toolbar_view.tap do |view|
        view.add_top_bar(header_bar)
        view.content = view_stack
        view.add_bottom_bar(view_switcher_bar)

        header_bar.tap do |bar|
          bar.pack_start(cancel_button)
          bar.pack_end(create_button)
        end

        view_stack.tap do |stack|
          stack.add_titled_with_icon(
            new_page,
            'new',
            _('New'),
            'list-add-symbolic',
          )
          # TRANSLATORS BibTeX is a proper name
          stack.add_titled_with_icon(
            source_view,
            'from_bibtex',
            _('From BibTeX'),
            'document-new-symbolic',
          )
          stack.add_titled_with_icon(
            doi_page,
            'from_doi',
            _('From DOI'),
            'document-new-symbolic',
          )
        end
      end

      new_page.tap do |page|
        page.append(new_list_box)

        new_list_box.tap do |list|
          list.append(entry_row)
          list.append(combo_row)
        end
      end

      doi_page.add(doi_row)

      dialog.default_widget = create_button

      cancel_button.signal_connect('clicked') { dialog.close }
      create_button.signal_connect('clicked') { on_create_clicked }
      entry_row.signal_connect('notify::text') { update_create_button }
      doi_row.signal_connect('notify::text') { update_create_button }
      view_stack.signal_connect('notify::visible-child') { update_create_button }
      source_view.buffer = bibtex_buffer
      source_view.buffer.signal_connect('changed') { update_create_button }
      source_view.add_controller(focus_controller)

      dialog.signal_connect('closed') { answer(nil) }

      update_style_scheme
      Adwaita::StyleManager.default.signal_connect('notify::dark') { update_style_scheme }

      source_view.buffer.text = "#{_('Insert a BibTeX citation here')}\n\n#{SAMPLE_ENTRY}"

      update_create_button
    end
  end

  def dialog
    @dialog ||= Adwaita::Dialog.new.tap do |widget|
      widget.width_request = 360
      widget.height_request = 360
      widget.content_height = 270
      widget.title = _('Add new Entry')
    end
  end

  def toolbar_view = @toolbar_view ||= Adwaita::ToolbarView.new

  def header_bar
    @header_bar ||= Adwaita::HeaderBar.new.tap do |bar|
      bar.show_end_title_buttons = false
      bar.show_title = false
    end
  end

  def cancel_button
    @cancel_button ||= Gtk::Button.new.tap do |button|
      button.label = _('_Cancel')
      button.use_underline = true
      button.can_shrink = true
    end
  end

  def create_button
    @create_button ||= Gtk::Button.new.tap do |button|
      button.label = _('C_reate')
      button.use_underline = true
      button.sensitive = false
      button.can_shrink = true
      button.add_css_class('suggested-action')
    end
  end

  def view_stack
    @view_stack ||= Adwaita::ViewStack.new.tap do |stack|
      stack.vexpand = true
    end
  end

  def view_switcher_bar
    @view_switcher_bar ||= Adwaita::ViewSwitcherBar.new.tap do |bar|
      bar.stack = view_stack
      bar.reveal = true
      bar.valign = :end
    end
  end

  def new_page
    @new_page ||= Gtk::Box.new(:vertical, 6).tap do |box|
      box.margin_start = 12
      box.margin_end = 12
      box.margin_top = 12
      box.margin_bottom = 12
    end
  end

  def new_list_box
    @new_list_box ||= Gtk::ListBox.new.tap do |list|
      list.selection_mode = :none
      list.add_css_class('boxed-list')
    end
  end

  def entry_row
    @entry_row ||= Adwaita::EntryRow.new.tap do |row|
      row.title = _('Citation Key')
      row.activates_default = true
    end
  end

  def combo_row
    @combo_row ||= Adwaita::ComboRow.new.tap do |row|
      row.title = _('Citation Type')
      row.model = Gtk::StringList.new(EntryType::ALL.map(&:translated_name))
      row.selected = DEFAULT_TYPE_INDEX
    end
  end

  # upstream's GtkSourceView `source_view`
  def source_view
    @source_view ||= GtkSource::View.new.tap do |view|
      view.hexpand = true
      view.wrap_mode = :word
      view.add_css_class('newentrydialog')
    end
  end

  # upstream's constructed: a bibtex-language buffer with syntax highlighting
  # on and matching-bracket highlighting off.
  def bibtex_buffer
    @bibtex_buffer ||= GtkSource::Buffer.new(GtkSource::LanguageManager.default.get_language('bibtex')).tap do |buffer|
      buffer.highlight_syntax = true
      buffer.highlight_matching_brackets = false
    end
  end

  def doi_page
    @doi_page ||= Adwaita::PreferencesGroup.new.tap do |group|
      group.margin_start = 12
      group.margin_end = 12
      group.margin_top = 12
      group.margin_bottom = 12
    end
  end

  def doi_row
    @doi_row ||= Adwaita::EntryRow.new.tap do |row|
      # TRANSLATORS DOI is a proper name
      row.title = _('DOI')
      row.activates_default = true
    end
  end

  private

    # upstream's NewEntryDialog::update_style_scheme
    def update_style_scheme
      scheme = GtkSource::StyleSchemeManager.default.get_scheme(dark_scheme_name)
      source_view.buffer.style_scheme = scheme
    end

    def dark_scheme_name
      if Adwaita::StyleManager.default.dark?
        'Adwaita-dark'
      else
        'Adwaita'
      end
    end

    # The instructions go away the first time the editor is focused.
    def focus_controller
      @focus_controller ||= Gtk::EventControllerFocus.new.tap do |controller|
        controller.signal_connect('enter') do
          unless @text_removed
            @text_removed = true
            source_view.buffer.text = ''
          end
        end
      end
    end

    def on_create_clicked
      case view_stack.visible_child_name
      when 'new' then answer("@#{selected_type.id}{#{entry_row.text},}")
      when 'from_bibtex' then answer(source_view.buffer.text)
      else answer_from_doi
      end
    end

    def selected_type = EntryType.from_index(combo_row.selected)

    # The DOI lookup is a network request, so it runs on a thread and comes
    # back on the main loop.
    def answer_from_doi
      doi_row.text.then do |doi|
        create_button.sensitive = false

        Thread.new do
          begin
            bibtex = Utils.bib_from_doi(doi)
            GLib::Idle.add do
              answer(bibtex)
              dialog.close

              false
            end
          rescue StandardError => e
            GLib::Idle.add do
              warn "Could not get valid entry from dialog: #{e.message}"
              answer(nil)
              dialog.close

              false
            end
          end
        end
      end
    end

    def answer(data)
      unless @answered || @on_answer.nil?
        @answered = true
        @on_answer.call(data)
      end

      unless data.nil?
        dialog.close
      end
    end

    # upstream's update_create_button
    def update_create_button
      create_button.sensitive = !visible_page_empty?
    end

    def visible_page_empty?
      case view_stack.visible_child_name
      when 'new' then entry_row.text.empty?
      when 'from_bibtex' then source_view.buffer.text.empty?
      else doi_row.text.empty?
      end
    end

end
