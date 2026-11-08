# frozen_string_literal: true

require 'adwaita'
require 'gtksourceview5'

require_relative 'citation_format'
require_relative 'entry_type'
require_relative 'i18n'
require_relative 'settings'

# CitationPreview shows the selected citation rendered from a template, with a
# drop down for the template and one for the output format.
#
# Ported from upstream's src/citation_preview.rs and
# data/resources/ui/citation_preview.ui: the Template enum and its fourteen
# built-in templates (an article and a book variant of each of MLA, AMS, APA,
# Chicago, Harvard and Vancouver, plus the custom pair), the revealer holding
# the custom template editor, the help popover, and the copy button. The
# rendered citation shows in a real GtkSource::View (source_view), with the
# markdown or latex language and the Adwaita/Adwaita-dark style scheme
# switched with the dark preference, matching upstream's set_buffer and
# update_style_scheme.
#
class CitationPreview
  include I18n::Translatable

  AMS_ARTICLE_TEMPLATE = '%AUTHOR%, *%TITLE%*, %JOURNAL% **%VOLUME%** (%YEAR%), no. %NUMBER%, %PAGES%.'
  MLA_ARTICLE_TEMPLATE = '%AUTHOR%. “%TITLE%.” *%JOURNAL%*, vol. %VOLUME%, no. %NUMBER%, %MONTH% %YEAR%, pp. %PAGES%.'
  APA_ARTICLE_TEMPLATE = '%AUTHOR%. (%YEAR%). %TITLE%. *%JOURNAL%*, *%VOLUME%*(%NUMBER%), %PAGES%.'
  CHICAGO_ARTICLE_TEMPLATE = '%AUTHOR%. "%TITLE%". *%JOURNAL%* %VOLUME%, no. %NUMBER% (%YEAR%): %PAGES%.'
  HARVARD_ARTICLE_TEMPLATE = '%AUTHOR% (%YEAR%) ‘%TITLE%’, *%JOURNAL%*, %VOLUME%(%NUMBER%), p%PAGES%.'
  VANCOUVER_ARTICLE_TEMPLATE = '%AUTHOR%. %TITLE%. %JOURNAL%. %YEAR% %MONTH% %DAY%; %VOLUME%(%NUMBER%):%PAGES%.'

  AMS_BOOK_TEMPLATE = '%AUTHOR%, *%TITLE%*, %PUBLISHER%, %CITY%, %YEAR%.'
  MLA_BOOK_TEMPLATE = '%AUTHOR%. *%TITLE%*. %PUBLISHER%, %YEAR%.'
  APA_BOOK_TEMPLATE = '%AUTHOR%. (%YEAR%). *%TITLE%*. (Vol. %VOLUME%). %PUBLISHER%.'
  CHICAGO_BOOK_TEMPLATE = '%AUTHOR%. *%TITLE%*. Vol. %VOLUME%. %PUBLISHER%, %YEAR%.'
  HARVARD_BOOK_TEMPLATE = '%AUTHOR%, (%YEAR%) *%TITLE%*. %VOLUME%. %PUBLISHER%.'
  VANCOUVER_BOOK_TEMPLATE = '%AUTHOR%: %TITLE%. %PUBLISHER%; %YEAR% %MONTH% %DAY%.'

  # The Template enum, in its own order: the order the drop down lists and the
  # "template-kind" setting stores.
  TEMPLATES = [
    ['Mla', MLA_ARTICLE_TEMPLATE, MLA_BOOK_TEMPLATE],
    ['Ams', AMS_ARTICLE_TEMPLATE, AMS_BOOK_TEMPLATE],
    ['Apa', APA_ARTICLE_TEMPLATE, APA_BOOK_TEMPLATE],
    ['Chicago', CHICAGO_ARTICLE_TEMPLATE, CHICAGO_BOOK_TEMPLATE],
    ['Harvard', HARVARD_ARTICLE_TEMPLATE, HARVARD_BOOK_TEMPLATE],
    ['Vancouver', VANCOUVER_ARTICLE_TEMPLATE, VANCOUVER_BOOK_TEMPLATE],
    ['Custom', nil, nil],
  ].freeze

  CUSTOM_INDEX = TEMPLATES.length - 1

  # The entry types whose citation reads as a book rather than an article,
  # from upstream's CitationPreview::use_book_format.
  BOOK_TYPES = %w[book inproceedings booklet inbook incollection].freeze

  def initialize(settings:, on_toast:)
    @settings = settings
    @on_toast = on_toast
    @entry = nil
    @handler = nil
  end

  def build
    bin.tap do |root|
      root.child = box

      box.tap do |container|
        container.append(template_revealer)
        container.append(citation_group)

        template_revealer.child = citation_fmt_group

        citation_fmt_group.tap do |group|
          group.header_suffix = help_button
          group.add(custom_template_window)

          custom_template_window.child = text_view
        end

        citation_group.tap do |group|
          group.header_suffix = drop_down_box
          group.add(citation_window)

          drop_down_box.tap do |suffix|
            suffix.append(template_drop_down)
            suffix.append(format_drop_down)
          end

          citation_window.child = overlay

          overlay.tap do |over|
            over.child = source_view
            over.add_overlay(copy_button)
          end
        end
      end

      help_popover.child = help_box

      help_box.tap do |content|
        content.append(help_title)
        content.append(help_label)
        content.append(help_example_title)
        content.append(help_window)

        help_window.child = help_source_view
      end

      help_source_view.buffer.text = AMS_ARTICLE_TEMPLATE

      copy_button.signal_connect('clicked') { on_copy_clicked }

      template_drop_down.signal_connect('notify::selected') do
        self.template_kind = template_drop_down.selected
      end

      format_drop_down.signal_connect('notify::selected') do
        self.format_kind = CitationFormat.from_index(format_drop_down.selected)
      end

      text_view.buffer.signal_connect('changed') { on_custom_template_changed }

      template_drop_down.selected = @settings['template-kind']
      format_drop_down.selected = @settings['format-kind']

      reset_source_buffer
      update_style_scheme
      Adwaita::StyleManager.default.signal_connect('notify::dark') { update_style_scheme }

      update_citation
    end
  end

  def widget = bin

  # upstream's CitationPreview::set_entry
  def entry=(entry)
    unless entry == @entry
      disconnect_entry
      @entry = entry
      connect_entry

      unless entry.nil?
        text_view.buffer.text = stored_custom_template
      end

      update_citation
    end
  end

  def template_kind = @template_kind ||= @settings['template-kind']

  def template_kind=(kind)
    unless kind == template_kind
      @template_kind = kind
      @settings['template-kind'] = kind
      template_revealer.reveal_child = kind == CUSTOM_INDEX
      update_citation
    end
  end

  def format_kind = @format_kind ||= CitationFormat.from_index(@settings['format-kind'])

  def format_kind=(kind)
    unless kind == format_kind
      @format_kind = kind
      @settings['format-kind'] = kind.index
      reset_source_buffer
      update_style_scheme
      update_citation
    end
  end

  # upstream's CitationPreview::template
  def template
    if template_kind == CUSTOM_INDEX
      text_view.buffer.text
    elsif use_book_format?
      TEMPLATES[template_kind][2]
    else
      TEMPLATES[template_kind][1]
    end
  end

  def bin = @bin ||= Adwaita::Bin.new
  def box = @box ||= Gtk::Box.new(:vertical, 6)

  def template_revealer
    @template_revealer ||= Gtk::Revealer.new.tap do |revealer|
      revealer.reveal_child = false
      revealer.transition_type = :slide_up
    end
  end

  def citation_fmt_group
    @citation_fmt_group ||= Adwaita::PreferencesGroup.new.tap do |group|
      group.title = _('Custom Citation Format')
    end
  end

  def help_button
    @help_button ||= Gtk::MenuButton.new.tap do |button|
      button.icon_name = 'dialog-question-symbolic'
      button.popover = help_popover
      button.tooltip_text = _('Help')
      button.add_css_class('flat')
    end
  end

  def custom_template_window
    @custom_template_window ||= Gtk::ScrolledWindow.new.tap do |window|
      window.margin_bottom = 24
      window.height_request = 90
      window.add_css_class('card')
    end
  end

  def text_view
    @text_view ||= Gtk::TextView.new.tap do |view|
      view.hexpand = true
      view.monospace = true
      view.wrap_mode = :word
      view.add_css_class('inline')
    end
  end

  def citation_group
    @citation_group ||= Adwaita::PreferencesGroup.new.tap do |group|
      group.title = _('Citation')
    end
  end

  def drop_down_box = @drop_down_box ||= Gtk::Box.new(:horizontal, 6)

  def template_drop_down
    @template_drop_down ||= Gtk::DropDown.new(Gtk::StringList.new(template_names), nil)
  end

  def format_drop_down
    @format_drop_down ||= Gtk::DropDown.new(
      Gtk::StringList.new(CitationFormat::ALL.map(&:translated_name)),
      nil,
    )
  end

  def citation_window
    @citation_window ||= Gtk::ScrolledWindow.new.tap do |window|
      window.height_request = 90
      window.add_css_class('card')
    end
  end

  def overlay = @overlay ||= Gtk::Overlay.new

  def copy_button
    @copy_button ||= Gtk::Button.new.tap do |button|
      button.halign = :end
      button.valign = :end
      button.icon_name = 'edit-copy-symbolic'
      button.margin_end = 6
      button.margin_start = 6
      button.margin_top = 6
      button.margin_bottom = 6
      button.tooltip_text = _('Copy')
      button.add_css_class('flat')
    end
  end

  # upstream's GtkSourceView `source_view`
  def source_view
    @source_view ||= GtkSource::View.new.tap do |view|
      view.hexpand = true
      view.wrap_mode = :word
      view.editable = false
      view.accepts_tab = false
      view.add_css_class('inline')
    end
  end

  def help_popover = @help_popover ||= Gtk::Popover.new

  def help_box
    @help_box ||= Gtk::Box.new(:vertical, 6).tap do |box|
      box.margin_start = 6
      box.margin_end = 6
      box.margin_top = 6
      box.margin_bottom = 6
    end
  end

  def help_title
    @help_title ||= Gtk::Label.new(_('Custom Template')).tap do |label|
      label.xalign = 0
      label.add_css_class('heading')
    end
  end

  def help_label
    @help_label ||= Gtk::Label.new.tap do |label|
      label.xalign = 0
      label.wrap = true
      label.wrap_mode = :word_char
      label.ellipsize = :none
      label.lines = 4
      label.use_markup = true
      label.max_width_chars = 40
      label.label = _(
        'Write a custom format for your citations, to insert the contents of a tag ' \
        '<tt>sometag</tt> write <tt>%SOMETAG%</tt>. Basic markdown syntax for bold ' \
        'and italics is supported. ',
      )
    end
  end

  def help_example_title
    @help_example_title ||= Gtk::Label.new(_('Example:')).tap do |label|
      label.xalign = 0
      label.add_css_class('heading')
    end
  end

  def help_window
    @help_window ||= Gtk::ScrolledWindow.new.tap do |window|
      window.height_request = 70
      window.width_request = 300
      window.max_content_width = 300
      window.hscrollbar_policy = :never
      window.add_css_class('card')
    end
  end

  def help_source_view
    @help_source_view ||= Gtk::TextView.new.tap do |view|
      view.hexpand = true
      view.wrap_mode = :word
      view.editable = false
      view.monospace = true
      view.add_css_class('inline')
    end
  end

  private

    def update_citation
      unless @entry.nil?
        source_view.buffer.text = @entry.format_citation(template.to_s, format_kind)
      end
    end

    # upstream's CitationPreview::set_buffer: a fresh GtkSource::Buffer with
    # the language matching format_kind, syntax highlighting on, matching
    # bracket highlighting off (upstream turns it off unconditionally too).
    def reset_source_buffer
      language = source_language_id.then { |id| id && GtkSource::LanguageManager.default.get_language(id) }
      GtkSource::Buffer.new(language).tap do |buffer|
        buffer.highlight_syntax = true
        buffer.highlight_matching_brackets = false
        source_view.buffer = buffer
      end
    end

    def source_language_id
      if format_kind.markdown?
        'markdown'
      elsif format_kind.latex?
        'latex'
      end
    end

    # upstream's CitationPreview::update_style_scheme
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

    def on_copy_clicked
      bin.display.clipboard.set(source_view.buffer.text)
      @on_toast.call(_('Copied'))
    end

    def on_custom_template_changed
      text_view.buffer.text.then do |custom|
        if use_book_format?
          @settings['custom-book-template'] = custom
        else
          @settings['custom-article-template'] = custom
        end
      end

      update_citation
    end

    def stored_custom_template
      if use_book_format?
        @settings['custom-book-template']
      else
        @settings['custom-article-template']
      end
    end

    def use_book_format?
      if @entry.nil?
        false
      else
        BOOK_TYPES.include?(@entry.entry_type.id)
      end
    end

    def connect_entry
      unless @entry.nil?
        @handler = @entry.signal_connect('notify') { update_citation }
      end
    end

    def disconnect_entry
      unless @entry.nil? || @handler.nil?
        @entry.signal_handler_disconnect(@handler)
        @handler = nil
      end
    end

    def template_names
      TEMPLATES.map do |name, _article, _book|
        case name
        when 'Mla', 'Ams', 'Apa'
          name.upcase
        when 'Chicago'
          _('Chicago')
        when 'Harvard'
          _('Harvard')
        when 'Vancouver'
          _('Vancouver')
        when 'Custom'
          # NOTE: Custom as in Custom Template.
          _('Custom…')
        end
      end
    end

end
