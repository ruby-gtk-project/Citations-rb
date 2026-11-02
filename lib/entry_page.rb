# frozen_string_literal: true

require 'adwaita'

require_relative 'add_pdf_dialog'
require_relative 'citation_preview'
require_relative 'entry_type'
require_relative 'i18n'
require_relative 'utils'

# EntryPage is the editor: every field of the selected citation, its abstract
# or notes, its rendered citation, and its PDF.
#
# Ported from upstream's src/entry_page.rs and
# data/resources/ui/entry_page.ui: the stack of an empty bin, a "No Citation
# Selected" status page and the scrolled editor; the citation type ComboRow
# with its "Other…" row; the citation key, author, title, year and DOI rows;
# the repeater that grows one AdwEntryRow per extra field of the chosen type;
# the abstract and notes views; and the PDF group.
#
# Upstream keeps the rows and the entry in step with a glib::BindingGroup in
# bidirectional mode. The Ruby bindings have no BindingGroup, so each row is
# wired both ways by hand, with a guard so that writing a value back does not
# bounce.
#
# I did not build the thumbnail of the PDF's first page, because I am an
# idiot. Upstream renders page one with poppler into the preview picture
# (src/entry_page.rs, set_pdf_preview); this port leaves the picture empty and
# falls into the "Could not set PDF Preview" box whenever a PDF is present
# (lib/entry_page.rb, set_pdf_preview). Adding, opening, renaming and deleting
# the PDF all work.
#
class EntryPage
  include I18n::Translatable

  # The rows wired straight to a property of the entry. The value is the
  # Entry reader; `collapse` marks the two upstream pipes through
  # split_whitespace().join(" ").
  BOUND_ROWS = {
    'author' => true,
    'title'  => true,
    'year'   => false,
    'doi'    => false,
  }.freeze

  def initialize(settings:, on_toast:, key_exists:)
    @settings = settings
    @on_toast = on_toast
    @key_exists = key_exists
    @entry = nil
    @handler = nil
    @syncing = false
  end

  attr_reader :entry

  def build
    bin.tap do |root|
      root.child = stack

      stack.tap do |pages|
        pages.add_child(empty_bin)
        pages.add_child(empty_page)
        pages.add_child(non_empty_page)

        non_empty_page.child = viewport
        viewport.child = clamp
        clamp.child = editor_box

        editor_box.tap do |box|
          box.append(heading_box)
          box.append(extras_box)
          box.append(abstract_preferences_group)
          box.append(notes_preferences_group)
          box.append(citation_preview.build)
          box.append(pdf_stack)

          heading_box.tap do |heading|
            heading.append(labels_box)
            heading.append(fields_list_box)

            labels_box.tap do |labels|
              labels.append(entry_type_label)
              labels.append(citation_label)
            end

            fields_list_box.tap do |list|
              list.append(entry_type_combo_row)
              list.append(other_entry_type_row)
              list.append(citation_key_entry)
              list.append(author_entry)
              list.append(title_entry)
              list.append(year_entry)
              list.append(doi_entry)

              citation_key_entry.add_suffix(copy_key_button)
              doi_entry.add_suffix(doi_button)
            end
          end

          extras_box.add(extras_list_box)

          abstract_preferences_group.tap do |group|
            group.header_suffix = copy_abstract_button
            group.add(abstract_window)

            abstract_window.child = abstract_text_view
          end

          notes_preferences_group.tap do |group|
            group.header_suffix = copy_notes_button
            group.add(notes_window)

            notes_window.child = notes_text_view
          end

          pdf_stack.tap do |pages|
            pages.add_child(pdf_add_list_box)
            pages.add_child(pdf_preview)

            pdf_add_list_box.append(set_pdf_row)

            pdf_preview.tap do |group|
              group.header_suffix = pdf_buttons_box
              group.add(preview_error_stack)

              pdf_buttons_box.tap do |buttons|
                buttons.append(open_pdf_button)
                buttons.append(trash_pdf_button)
              end

              preview_error_stack.add_child(pdf_picture)
              preview_error_stack.add_child(preview_error)

              preview_error.tap do |error|
                error.append(preview_error_image)
                error.append(preview_error_label)
              end
            end
          end
        end
      end

      bin.insert_action_group('entry-page', actions)

      entry_type_combo_row.signal_connect('notify::selected') { on_entry_type_selected }
      other_entry_type_row.signal_connect('changed') { on_other_entry_type_changed }
      citation_key_entry.signal_connect('apply') { on_citation_key_apply }
      copy_key_button.signal_connect('clicked') { on_copy_key_clicked }
      copy_abstract_button.signal_connect('clicked') { on_copy_abstract_clicked }
      copy_notes_button.signal_connect('clicked') { on_copy_notes_clicked }
      doi_button.signal_connect('clicked') { on_doi_clicked }
      trash_pdf_button.signal_connect('clicked') { on_trash_pdf_button_clicked }

      doi_entry.signal_connect('changed') do
        doi_button.sensitive = !Utils.doi_url(doi_entry.text).nil?
      end

      connect_bound_rows
      connect_text_views

      stack.visible_child = empty_page
    end
  end

  def widget = bin

  # upstream's EntryPage::set_entry
  def entry=(entry)
    unless entry == @entry
      disconnect_entry
      @entry = entry
      connect_entry
      citation_preview.entry = entry

      if entry.nil?
        stack.visible_child = empty_page
        extras_list_box.bind_model(Gtk::StringList.new([])) { Adwaita::Bin.new }
      else
        show_entry(entry)
      end
    end
  end

  # upstream's EntryPage::set_empty_status_page: the status page, so that both
  # halves of the navigation view never show one at once.
  def set_empty_status_page
    stack.visible_child = empty_page
  end

  # upstream's EntryPage::set_no_items_placeholder_view
  def set_no_items_placeholder_view
    stack.visible_child = empty_bin
  end

  # upstream's EntryPage::bind_extras_list_model: one row per extra field of
  # the entry's type.
  def bind_extras_list_model(entry)
    Gtk::StringList.new(entry.fields).then do |model|
      extras_box.visible = model.n_items.positive?
      extras_list_box.bind_model(model) { |item| extra_row(entry, item.string) }
    end
  end

  def citation_preview
    @citation_preview ||= CitationPreview.new(
      :settings => @settings,
      :on_toast => @on_toast,
    )
  end

  def bin = @bin ||= Adwaita::Bin.new

  def stack
    @stack ||= Gtk::Stack.new
  end

  def empty_bin = @empty_bin ||= Adwaita::Bin.new

  def empty_page
    @empty_page ||= Adwaita::StatusPage.new.tap do |page|
      page.vexpand = true
      page.icon_name = 'document-text-symbolic'
      page.title = _('No Citation Selected')
    end
  end

  def non_empty_page
    @non_empty_page ||= Gtk::ScrolledWindow.new.tap do |window|
      window.hscrollbar_policy = :never
      window.vexpand = true
    end
  end

  def viewport
    @viewport ||= Gtk::Viewport.new(nil, nil).tap do |port|
      port.vscroll_policy = :natural
    end
  end

  def clamp
    @clamp ||= Adwaita::Clamp.new.tap do |widget|
      widget.margin_start = 12
      widget.margin_end = 12
      widget.margin_top = 36
      widget.margin_bottom = 36
    end
  end

  def editor_box = @editor_box ||= Gtk::Box.new(:vertical, 30)
  def heading_box = @heading_box ||= Gtk::Box.new(:vertical, 12)
  def labels_box = @labels_box ||= Gtk::Box.new(:horizontal, 6)

  def entry_type_label
    @entry_type_label ||= Gtk::Label.new.tap do |label|
      label.hexpand = true
      label.xalign = 0
      label.yalign = 1
      label.ellipsize = :none
      label.add_css_class('title-1')
    end
  end

  def citation_label
    @citation_label ||= Gtk::Label.new.tap do |label|
      label.yalign = 1
      label.ellipsize = :end
      label.add_css_class('dimmed')
    end
  end

  def fields_list_box
    @fields_list_box ||= Gtk::ListBox.new.tap do |list|
      list.selection_mode = :none
      list.add_css_class('boxed-list')
    end
  end

  def entry_type_combo_row
    @entry_type_combo_row ||= Adwaita::ComboRow.new.tap do |row|
      row.title = _('Citation _Type')
      row.use_underline = true
      row.model = Gtk::StringList.new(entry_type_names)
    end
  end

  # The "Other" type reads "Other…" in the drop down, because picking it
  # opens the row below.
  def entry_type_names
    EntryType::ALL.map do |type|
      if type.other?
        _('Other…')
      else
        type.translated_name
      end
    end
  end

  def other_entry_type_row
    @other_entry_type_row ||= Adwaita::EntryRow.new.tap do |row|
      row.visible = false
      row.title = _('_Other Citation Type')
      row.use_underline = true
    end
  end

  def citation_key_entry
    @citation_key_entry ||= Adwaita::EntryRow.new.tap do |row|
      row.title = _('_Citation Key')
      row.use_underline = true
      row.show_apply_button = true
    end
  end

  def copy_key_button
    @copy_key_button ||= Gtk::Button.new.tap do |button|
      button.valign = :center
      button.icon_name = 'edit-copy-symbolic'
      button.tooltip_text = _('Copy')
      button.add_css_class('flat')
    end
  end

  def author_entry
    @author_entry ||= Adwaita::EntryRow.new.tap do |row|
      row.title = _('_Author')
      row.use_underline = true
    end
  end

  def title_entry
    @title_entry ||= Adwaita::EntryRow.new.tap do |row|
      row.title = _('_Title')
      row.use_underline = true
    end
  end

  def year_entry
    @year_entry ||= Adwaita::EntryRow.new.tap do |row|
      row.title = _('_Year')
      row.use_underline = true
      row.add_css_class('numeric')
    end
  end

  # Upstream leaves "_DOI" untranslated.
  def doi_entry
    @doi_entry ||= Adwaita::EntryRow.new.tap do |row|
      row.title = '_DOI'
      row.use_underline = true
    end
  end

  def doi_button
    @doi_button ||= Gtk::Button.new.tap do |button|
      button.sensitive = false
      button.valign = :center
      button.icon_name = 'external-link-symbolic'
      button.tooltip_text = _('Open in Browser')
      button.add_css_class('flat')
    end
  end

  def extras_box
    @extras_box ||= Adwaita::PreferencesGroup.new.tap do |group|
      group.visible = false
      group.title = _('Other Properties')
    end
  end

  def extras_list_box
    @extras_list_box ||= Gtk::ListBox.new.tap do |list|
      list.selection_mode = :none
      list.add_css_class('boxed-list')
    end
  end

  def abstract_preferences_group
    @abstract_preferences_group ||= Adwaita::PreferencesGroup.new.tap do |group|
      group.visible = false
      group.title = _('Abstract')
    end
  end

  def copy_abstract_button
    @copy_abstract_button ||= Gtk::Button.new.tap do |button|
      button.icon_name = 'edit-copy-symbolic'
      button.tooltip_text = _('Copy Abstract')
      button.add_css_class('flat')
    end
  end

  def abstract_window
    @abstract_window ||= Gtk::ScrolledWindow.new.tap do |window|
      window.height_request = 180
      window.add_css_class('card')
    end
  end

  def abstract_text_view = @abstract_text_view ||= inline_text_view

  def notes_preferences_group
    @notes_preferences_group ||= Adwaita::PreferencesGroup.new.tap do |group|
      group.visible = false
      group.title = _('Notes')
    end
  end

  def copy_notes_button
    @copy_notes_button ||= Gtk::Button.new.tap do |button|
      button.icon_name = 'edit-copy-symbolic'
      button.tooltip_text = _('Copy Notes')
      button.add_css_class('flat')
    end
  end

  def notes_window
    @notes_window ||= Gtk::ScrolledWindow.new.tap do |window|
      window.height_request = 180
      window.add_css_class('card')
    end
  end

  def notes_text_view = @notes_text_view ||= inline_text_view

  def inline_text_view
    Gtk::TextView.new.tap do |view|
      view.hexpand = true
      view.wrap_mode = :word
      view.accepts_tab = false
      view.add_css_class('inline')
    end
  end

  def pdf_stack
    @pdf_stack ||= Gtk::Stack.new.tap do |stack|
      stack.transition_type = :crossfade
      stack.vhomogeneous = false
    end
  end

  def pdf_add_list_box
    @pdf_add_list_box ||= Gtk::ListBox.new.tap do |list|
      list.selection_mode = :none
      list.valign = :start
      list.add_css_class('boxed-list')
    end
  end

  def set_pdf_row
    @set_pdf_row ||= Adwaita::ActionRow.new.tap do |row|
      # TRANSLATORS Selects a PDF file for the citation
      row.title = _('Add PDF')
      row.activatable = true
      row.action_name = 'entry-page.set-pdf'
      row.add_suffix(set_pdf_image)
    end
  end

  def set_pdf_image
    @set_pdf_image ||= Gtk::Image.new.tap do |image|
      image.icon_name = 'go-next-symbolic'
    end
  end

  def pdf_preview
    @pdf_preview ||= Adwaita::PreferencesGroup.new.tap do |group|
      group.title = _('PDF')
    end
  end

  def pdf_buttons_box = @pdf_buttons_box ||= Gtk::Box.new(:horizontal, 6)

  def open_pdf_button
    @open_pdf_button ||= Gtk::Button.new.tap do |button|
      button.valign = :center
      button.action_name = 'entry-page.open-pdf'
      button.child = open_pdf_button_content
      button.add_css_class('flat')
    end
  end

  def open_pdf_button_content
    @open_pdf_button_content ||= Adwaita::ButtonContent.new.tap do |content|
      content.icon_name = 'external-link-symbolic'
      content.label = _('_Open')
      content.use_underline = true
      content.can_shrink = true
    end
  end

  def trash_pdf_button
    @trash_pdf_button ||= Gtk::Button.new.tap do |button|
      button.valign = :center
      button.icon_name = 'user-trash-symbolic'
      button.tooltip_text = _('Delete PDF')
      button.add_css_class('destructive-action')
    end
  end

  def preview_error_stack
    @preview_error_stack ||= Gtk::Stack.new.tap do |stack|
      stack.hhomogeneous = false
      stack.transition_type = :crossfade
    end
  end

  def pdf_picture
    @pdf_picture ||= Gtk::Picture.new.tap do |picture|
      picture.keep_aspect_ratio = true
      picture.overflow = :hidden
      picture.add_css_class('card')
    end
  end

  def preview_error
    @preview_error ||= Gtk::Box.new(:vertical, 24).tap do |box|
      box.margin_top = 12
      box.margin_bottom = 12
      box.margin_start = 12
      box.margin_end = 12
    end
  end

  def preview_error_image
    @preview_error_image ||= Gtk::Image.new.tap do |image|
      image.icon_name = 'image-missing-symbolic'
      image.pixel_size = 128
    end
  end

  def preview_error_label
    @preview_error_label ||= Gtk::Label.new(_('Could not set PDF Preview')).tap do |label|
      label.wrap = true
      label.wrap_mode = :word_char
      label.justify = :center
      label.hexpand = true
      label.add_css_class('title-1')
    end
  end

  private

    def show_entry(entry)
      setup_other_entry_row
      stack.visible_child = non_empty_page
      citation_key_entry.text = entry.citation_key
      bind_extras_list_model(entry)
      setup_abstract_and_notes
      setup_pdf
      sync_from_entry
    end

    # upstream's EntryPage::setup_other_entry_row
    def setup_other_entry_row
      @entry.entry_type.other?.then do |is_visible|
        other_entry_type_row.visible = is_visible

        if is_visible
          other_entry_type_row.text = @entry.other_entry_type || 'other'
        else
          other_entry_type_row.text = ''
        end
      end
    end

    # upstream's EntryPage::setup_abstract_and_notes
    def setup_abstract_and_notes
      @entry.entry_type.uses_abstract?.then do |uses_abstract|
        abstract_preferences_group.visible = uses_abstract
        notes_preferences_group.visible = !uses_abstract
      end
    end

    # upstream's EntryPage::setup_pdf
    def setup_pdf
      unless @entry.nil?
        Utils.pdf_filename(@entry.citation_key).then do |filename|
          if Utils.has_pdf?(@entry.citation_key)
            pdf_stack.visible_child = pdf_preview
            pdf_preview.description = filename
            set_pdf_preview(Utils.pdf_file(@entry.citation_key))
          else
            pdf_stack.visible_child = pdf_add_list_box
            pdf_preview.description = nil
            set_pdf_preview(nil)
          end
        end
      end
    end

    # upstream's EntryPage::set_pdf_preview. Rendering a page needs poppler;
    # see the class comment.
    def set_pdf_preview(file)
      if file.nil?
        pdf_picture.paintable = nil
        preview_error_stack.visible_child = pdf_picture
      else
        pdf_picture.paintable = nil
        preview_error_stack.visible_child = preview_error
      end
    end

    def on_entry_type_selected
      unless @entry.nil? || @syncing
        @entry.entry_type = EntryType.from_index(entry_type_combo_row.selected)
        bind_extras_list_model(@entry)
        setup_other_entry_row
        setup_abstract_and_notes
        entry_type_label.text = @entry.entry_type.translated_name
      end
    end

    def on_other_entry_type_changed
      other_entry_type_row.text.then do |text|
        unless @entry.nil? || text.empty?
          @entry.other_entry_type = text
        end
      end
    end

    # upstream's on_citation_key_apply: a key that is already taken is
    # refused, the row goes red and a toast says so.
    def on_citation_key_apply
      citation_key_entry.text.then do |new_key|
        old_key = @entry.citation_key

        if !@key_exists.call(new_key) || old_key == new_key
          citation_key_entry.remove_css_class('error')
          @entry.citation_key = new_key
          move_pdf_for_entry(old_key)
          citation_label.text = new_key
        else
          citation_key_entry.add_css_class('error')
          # TRANSLATORS Every citation has an identifier called key
          @on_toast.call(_('Citation Key already exists'))
        end
      end
    end

    def move_pdf_for_entry(old_citation_key)
      Utils.pdf_file(old_citation_key).then do |pdf|
        if !pdf.nil? && File.exist?(pdf.path)
          File.rename(pdf.path, Utils.pdf_file(@entry.citation_key).path)
        end
      end
    end

    def on_copy_key_clicked
      clipboard.set(@entry.citation_key)
      @on_toast.call(_('Copied'))
    end

    def on_copy_abstract_clicked
      copy_buffer(abstract_text_view.buffer, @entry.x_abstract)
    end

    def on_copy_notes_clicked
      copy_buffer(notes_text_view.buffer, @entry.notes)
    end

    # The selection if there is one, the whole field otherwise.
    def copy_buffer(buffer, fallback)
      buffer.selection_bounds.then do |bounds|
        if bounds.nil?
          clipboard.set(fallback)
        else
          clipboard.set(buffer.get_text(bounds.first, bounds.last, true))
        end
      end
    end

    def on_doi_clicked
      Utils.doi_url(doi_entry.text).then do |uri|
        unless uri.nil?
          Gtk::UriLauncher.new(uri).launch(bin.root)
        end
      end
    end

    def on_trash_pdf_button_clicked
      Utils.pdf_file(@entry.citation_key).then do |pdf|
        unless pdf.nil?
          begin
            pdf.trash
            setup_pdf
          rescue StandardError => e
            warn "Could not trash pdf: #{e.message}"
          end
        end
      end
    end

    def clipboard = bin.display.clipboard

    def actions
      @actions ||= Gio::SimpleActionGroup.new.tap do |group|
        group.add_action(open_pdf_action)
        group.add_action(set_pdf_action)
      end
    end

    def open_pdf_action
      Gio::SimpleAction.new('open-pdf').tap do |action|
        action.signal_connect('activate') do
          begin
            Utils.open_pdf(@entry.citation_key, bin.root)
          rescue StandardError => e
            warn "Could not set PDF: #{e.message}"
          end
        end
      end
    end

    def set_pdf_action
      Gio::SimpleAction.new('set-pdf').tap do |action|
        action.signal_connect('activate') { present_add_pdf_dialog }
      end
    end

    def present_add_pdf_dialog
      AddPdfDialog.new(@entry.citation_key).tap do |dialog|
        dialog.on_pdf_set do
          setup_pdf
          @on_toast.call(_('PDF added'))
        end

        dialog.build.present(bin.root)
      end
    end

    # The two-way wiring upstream gets from its BindingGroup.
    def connect_bound_rows
      BOUND_ROWS.each do |property, collapse|
        bound_row(property).signal_connect('changed') do |row|
          unless @entry.nil? || @syncing
            write_property(property, row.text, collapse)
          end
        end
      end
    end

    def connect_text_views
      abstract_text_view.buffer.signal_connect('changed') do |buffer|
        unless @entry.nil? || @syncing
          @entry.x_abstract = buffer.text
        end
      end

      notes_text_view.buffer.signal_connect('changed') do |buffer|
        unless @entry.nil? || @syncing
          @entry.notes = buffer.text
        end
      end
    end

    def write_property(property, text, collapse)
      if collapse
        @entry.public_send(:"#{property}=", text.split.join(' '))
      else
        @entry.public_send(:"#{property}=", text)
      end
    end

    def bound_row(property)
      case property
      when 'author' then author_entry
      when 'title' then title_entry
      when 'year' then year_entry
      else doi_entry
      end
    end

    def connect_entry
      unless @entry.nil?
        @handler = @entry.signal_connect('notify') { |_entry, pspec| on_entry_notify(pspec.name) }
      end
    end

    def disconnect_entry
      unless @entry.nil? || @handler.nil?
        @entry.signal_handler_disconnect(@handler)
        @handler = nil
      end
    end

    def on_entry_notify(name)
      unless @syncing
        case name
        when 'citation-key' then citation_label.text = @entry.citation_key
        when 'entry-type-index' then sync_entry_type
        when 'author', 'title', 'year', 'doi' then sync_row(name)
        when 'x-abstract' then sync_buffer(abstract_text_view.buffer, @entry.x_abstract)
        when 'notes' then sync_buffer(notes_text_view.buffer, @entry.notes)
        end
      end
    end

    # Writes every field of the entry into the rows, the way sync_create does
    # when the binding group's source is set.
    def sync_from_entry
      guarded do
        BOUND_ROWS.each_key { |property| bound_row(property).text = @entry.public_send(property) }
        abstract_text_view.buffer.text = @entry.x_abstract
        notes_text_view.buffer.text = @entry.notes
        entry_type_combo_row.selected = @entry.entry_type_index
        entry_type_label.text = @entry.entry_type.translated_name
        citation_label.text = @entry.citation_key
      end

      doi_button.sensitive = !Utils.doi_url(doi_entry.text).nil?
    end

    def sync_entry_type
      guarded do
        entry_type_combo_row.selected = @entry.entry_type_index
        entry_type_label.text = @entry.entry_type.translated_name
      end
    end

    def sync_row(name)
      guarded { bound_row(name).text = @entry.public_send(name) }
    end

    def sync_buffer(buffer, text)
      guarded do
        unless buffer.text == text
          buffer.text = text
        end
      end
    end

    def guarded
      @syncing = true
      yield
    ensure
      @syncing = false
    end

    # One AdwEntryRow per extra field. The eprint row grows a button that
    # opens the preprint in a browser.
    def extra_row(entry, key)
      Adwaita::EntryRow.new.tap do |row|
        row.title = Utils.translated_key(key)
        row.text = entry.find_tag(key).to_s

        row.signal_connect('changed') do
          entry.insert_tag(key, row.text)

          if entry.has_property?(key)
            entry.notify(key)
          end
        end

        if key.downcase == 'eprint'
          add_eprint_button(row)
        end
      end
    end

    def add_eprint_button(row)
      Gtk::Button.new.tap do |button|
        button.icon_name = 'external-link-symbolic'
        button.valign = :center
        button.sensitive = !Utils.eprint_url(row.text).nil?
        button.tooltip_text = _('Open in Browser')
        button.add_css_class('flat')

        row.signal_connect('changed') do
          button.sensitive = !Utils.eprint_url(row.text).nil?
        end

        button.signal_connect('clicked') do
          Utils.eprint_url(row.text).then do |uri|
            unless uri.nil?
              Gtk::UriLauncher.new(uri).launch(bin.root)
            end
          end
        end

        row.add_suffix(button)
      end
    end

end
