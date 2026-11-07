# frozen_string_literal: true

require 'adwaita'
require 'fileutils'
require 'poppler'

require_relative 'i18n'
require_relative 'utils'

# AddPdfDialog adds a PDF to a citation, either by downloading it from a URL
# or by copying one from disk.
#
# Ported from upstream's src/add_pdf_dialog.rs and
# data/resources/ui/add_pdf_dialog.ui: an AdwDialog titled "Add PDF" with the
# URL row and its download button, the "Add From File…" button row, the osd
# progress bar that pulses while the download runs, the error labels shown
# when the bibliography folder is unknown, and the pdf-set signal the editor
# listens for.
#
# Upstream validates the downloaded bytes by handing them to poppler
# (poppler::Document::from_bytes); this port does the same, with the Poppler
# gem's Poppler::Document.new(data:), and rejects the download the same way
# on any Poppler::Error.
#
# Upstream downloads on a tokio runtime and awaits it; here the request runs on
# a Ruby thread and finishes on the main loop through GLib::Idle, so the dialog
# keeps pulsing rather than freezing.
#
class AddPdfDialog
  include I18n::Translatable

  PULSE_INTERVAL = 100

  def initialize(citation_key)
    @citation_key = citation_key
    @on_pdf_set = []
  end

  # upstream's "pdf-set" signal.
  def on_pdf_set(&block)
    @on_pdf_set << block
  end

  def build
    dialog.tap do |root|
      root.child = toolbar_view

      toolbar_view.tap do |view|
        view.add_top_bar(header_bar)
        view.content = toast_overlay

        toast_overlay.child = overlay

        overlay.tap do |over|
          over.child = scrolled_window
          over.add_overlay(progress_bar)
        end

        scrolled_window.child = clamp
        clamp.child = box

        box.tap do |content|
          content.append(error_box)
          content.append(description_label)
          content.append(list_box)

          error_box.tap do |errors|
            errors.append(error_title_label)
            errors.append(error_label)
          end

          list_box.tap do |list|
            list.append(pdf_url_entry_row)
            list.append(from_file_row)

            pdf_url_entry_row.add_suffix(download_button)
          end
        end
      end

      pdf_url_entry_row.signal_connect('entry-activated') { download }
      download_button.signal_connect('clicked') { download }
      from_file_row.signal_connect('activated') { set_pdf_from_file }

      show_folder_error
    end
  end

  def file_dialog
    @file_dialog ||= Gtk::FileDialog.new.tap do |chooser|
      chooser.title = _('Select a PDF')
      chooser.filters = pdf_filters
    end
  end

  def pdf_filters
    Gio::ListStore.new(Gtk::FileFilter.gtype).tap do |filters|
      filters.append(pdf_filter)
    end
  end

  def pdf_filter
    Gtk::FileFilter.new.tap do |filter|
      filter.name = _('PDF Files')
      filter.add_mime_type('application/pdf')
    end
  end

  def dialog
    @dialog ||= Adwaita::Dialog.new.tap do |widget|
      widget.width_request = 360
      widget.height_request = 270
      widget.content_width = 450
      widget.content_height = 270
      widget.title = _('Add PDF')
    end
  end

  def toolbar_view = @toolbar_view ||= Adwaita::ToolbarView.new
  def header_bar = @header_bar ||= Adwaita::HeaderBar.new
  def toast_overlay = @toast_overlay ||= Adwaita::ToastOverlay.new
  def overlay = @overlay ||= Gtk::Overlay.new

  def progress_bar
    @progress_bar ||= Gtk::ProgressBar.new.tap do |bar|
      bar.valign = :start
      bar.visible = false
      bar.add_css_class('osd')
    end
  end

  def scrolled_window = @scrolled_window ||= Gtk::ScrolledWindow.new

  def clamp
    @clamp ||= Adwaita::Clamp.new.tap do |widget|
      widget.vexpand = true
    end
  end

  def box
    @box ||= Gtk::Box.new(:vertical, 12).tap do |widget|
      widget.margin_top = 24
      widget.margin_bottom = 24
      widget.margin_start = 12
      widget.margin_end = 12
    end
  end

  def error_box
    @error_box ||= Gtk::Box.new(:horizontal, 0).tap do |widget|
      widget.visible = false
    end
  end

  def error_title_label
    @error_title_label ||= Gtk::Label.new(_('Bibliography folder not set')).tap do |label|
      label.xalign = 0
      label.add_css_class('error')
    end
  end

  def error_label
    @error_label ||= Gtk::Label.new.tap do |label|
      label.xalign = 0
      label.add_css_class('error')
    end
  end

  def description_label
    @description_label ||= Gtk::Label.new(
      _('Creates a copy of the selected pdf and stores it in your Bibliography folder.'),
    ).tap do |label|
      label.wrap_mode = :word_char
      label.wrap = true
      label.xalign = 0
      label.add_css_class('dimmed')
    end
  end

  def list_box
    @list_box ||= Gtk::ListBox.new.tap do |list|
      list.selection_mode = :none
      list.add_css_class('boxed-list-separate')
    end
  end

  def pdf_url_entry_row
    @pdf_url_entry_row ||= Adwaita::EntryRow.new.tap do |row|
      row.activatable = true
      row.title = _('Add From URL')
    end
  end

  def download_button
    @download_button ||= Gtk::Button.new.tap do |button|
      button.valign = :center
      button.icon_name = 'go-next-symbolic'
      button.tooltip_text = _('Download…')
      button.add_css_class('flat')
    end
  end

  def from_file_row
    @from_file_row ||= Adwaita::ButtonRow.new.tap do |row|
      row.title = _('Add From File…')
      row.start_icon_name = 'document-open-symbolic'
    end
  end

  private

    # upstream's constructed(): when there is no Documents directory there is
    # nowhere to put the PDF, and the dialog says so.
    def show_folder_error
      if Utils.bibliography_path.nil?
        error_box.visible = true
        # TRANSLATORS Do NOT translate {path}, it is a variable name
        error_label.label = _('Make sure that {path} exists')
          .sub('{path}', Utils.bibliography_path_string)
      end
    end

    def download
      Utils.absolute_url(pdf_url_entry_row.text).then do |url|
        if url.nil?
          download_failed('Not a URL')
        else
          start_download(url)
        end
      end
    end

    def start_download(url)
      download_button.sensitive = false
      progress_bar.visible = true
      start_pulsing

      Thread.new do
        begin
          finish_download(Utils.get(url))
        rescue StandardError => e
          finish_download(e)
        end
      end
    end

    def start_pulsing
      @pulse = GLib::Timeout.add(PULSE_INTERVAL) do
        progress_bar.pulse

        true
      end
    end

    def stop_pulsing
      unless @pulse.nil?
        GLib::Source.remove(@pulse)
        @pulse = nil
      end
    end

    # Back on the main loop with whatever the thread came up with.
    def finish_download(result)
      GLib::Idle.add do
        stop_pulsing
        progress_bar.visible = false

        if result.is_a?(String) && valid_pdf?(result)
          store(result)
        else
          download_failed(result)
        end

        false
      end
    end

    # upstream's download_pdf: validates the document by handing it to
    # poppler, same as set_pdf_preview does for an existing PDF.
    def valid_pdf?(bytes)
      Poppler::Document.new(data: bytes)
      true
    rescue Poppler::Error, GLib::Error => e
      warn "PDF validation failed: #{e.message}"
      false
    end

    def store(bytes)
      Utils.bibliography_path.then do |path|
        if path.nil?
          download_failed('No bibliography path')
        else
          FileUtils.mkdir_p(path)
          File.binwrite(File.join(path, Utils.pdf_filename(@citation_key)), bytes)
          pdf_url_entry_row.remove_css_class('error')
          emit_pdf_set
          dialog.close
        end
      end
    end

    def download_failed(result)
      warn "Could not download pdf: #{result}"
      pdf_url_entry_row.add_css_class('error')
      toast_overlay.add_toast(Adwaita::Toast.new(_('Could not download PDF')))
      download_button.sensitive = true
    end

    # upstream's set_pdf_from_file: copies the chosen PDF into the
    # bibliography folder under the citation key's name.
    def set_pdf_from_file
      Utils.bibliography_path.then do |path|
        unless path.nil?
          file_dialog.open(parent_window) do |chooser, result|
            copy_chosen_file(chooser, result, path)
          end
        end
      end
    end

    def copy_chosen_file(chooser, result, path)
      chooser.open_finish(result).then do |file|
        unless file.nil?
          FileUtils.mkdir_p(path)
          FileUtils.cp(file.path, File.join(path, Utils.pdf_filename(@citation_key)))
          emit_pdf_set
          dialog.close
        end
      end
    rescue StandardError => e
      warn "could not set pdf: #{e.message}"
      toast_overlay.add_toast(Adwaita::Toast.new(_('Could not set PDF')))
    end

    def parent_window = dialog.root

    def emit_pdf_set
      @on_pdf_set.each(&:call)
    end

end
