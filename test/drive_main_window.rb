# frozen_string_literal: true

# Drives the main window: the recent files screen, opening a bibliography, the
# sidebar, the editor, search, deletion with undo, saving, and the dialogs.

require 'tmpdir'
require 'fileutils'

ENV['XDG_CONFIG_HOME'] = Dir.mktmpdir('citations-config')
ENV['XDG_DATA_HOME'] = Dir.mktmpdir('citations-data')

$LOAD_PATH.unshift(File.expand_path('../lib', __dir__))

require 'main'
require_relative 'gtk_driver'

BIB = <<~BIBTEX
  @article{knuth1984literate,
    author = "Donald E. Knuth",
    title = "Literate Programming",
    year = "1984",
    journal = "The Computer Journal",
    volume = "27",
    number = "2",
    pages = "97--111",
  }

  @book{lamport1994latex,
    author = "Leslie Lamport",
    title = {\\LaTeX: A Document Preparation System},
    year = "1994",
    publisher = "Addison-Wesley",
  }
BIBTEX

dir = Dir.mktmpdir('citations-bib')
path = File.join(dir, 'library.bib')
File.write(path, BIB)

app = App.new(app_id: 'org.rubygtkproject.CitationsTest')

GtkDriver.drive(app, shots: 'tmp/shots', interval: 400) do |d, application|
  win = -> { application.windows.first }

  d.window { win.call.widget }

  d.step('the window comes up on the empty recent files state') do
    d.check('a window was built') { !win.call.nil? }
    d.check('the main stack shows the recent files screen') do
      win.call.main_stack.visible_child == win.call.recent_files_stack
    end
    d.check('the empty state page is showing') do
      win.call.recent_files_stack.visible_child == win.call.empty_state_stack_page
    end
    d.check('it offers to open a file') do
      win.call.no_recent_files_status_page.title == 'Open a BibTeX File'
    end
    d.check('new_entry, save and delete_entry are disabled') do
      %w[new_entry save delete_entry].none? { |name| win.call.widget.lookup_action(name).enabled? }
    end
    d.shot('01-empty-state')
  end

  d.step('opening a bibliography shows the split view') do
    win.call.open(Gio::File.new_for_path(path))
  end

  d.step('the sidebar lists both citations') do
    d.check('two entries were parsed') { win.call.bib.n_items == 2 }
    d.check('the split view is showing') do
      win.call.main_stack.visible_child == win.call.nav_split_view
    end
    d.check('save and new_entry are enabled now') do
      %w[new_entry save delete_entry].all? { |name| win.call.widget.lookup_action(name).enabled? }
    end
    d.check('new_biblio is disabled now') do
      !win.call.widget.lookup_action('new_biblio').enabled?
    end
    d.check('the path was recorded as a recent file') do
      win.call.settings.strv('recent-files').include?(path)
    end
    d.shot('02-opened')
  end

  d.step('selecting the first citation fills the editor') do
    win.call.entry_list.set_selected(0)
  end

  d.step('the editor shows the selected citation') do
    page = win.call.entry_page
    d.check('the editor has the entry') { page.entry&.citation_key == 'knuth1984literate' }
    d.check('the author row is filled') { page.author_entry.text == 'Donald E. Knuth' }
    d.check('the title row is filled') { page.title_entry.text == 'Literate Programming' }
    d.check('the year row is filled') { page.year_entry.text == '1984' }
    d.check('the type label names the type') { page.entry_type_label.text == 'Article' }
    d.check('the citation key label is set') { page.citation_label.text == 'knuth1984literate' }
    d.check('the extras group is visible') { page.extras_box.visible? }
    d.check('the abstract group is visible for an article') do
      page.abstract_preferences_group.visible?
    end
    d.check('the notes group is hidden for an article') do
      !page.notes_preferences_group.visible?
    end
    d.check('the secondary menu button became visible') do
      win.call.secondary_menu_button.visible?
    end
    d.check('the citation preview rendered the MLA template') do
      page.citation_preview.source_view.buffer.text.include?('Literate Programming')
    end
    d.shot('03-editor')
  end

  d.step('editing a row writes through to the model') do
    win.call.entry_page.title_entry.text = 'Literate Programming, Revisited'
  end

  d.step('the model took the edit') do
    d.check('the entry title changed') do
      win.call.bib.item(0).title == 'Literate Programming, Revisited'
    end
    d.check('the bibliography is marked modified') { win.call.bib.modified? }
    d.check('the preview followed the edit') do
      win.call.entry_page.citation_preview.source_view.buffer.text.include?('Revisited')
    end
  end

  d.step('searching filters the sidebar') do
    win.call.search_bar.search_mode = true
    win.call.search_entry.text = 'lamport'
  end

  d.step('only the matching citation is listed') do
    d.check('one row matches') { win.call.entry_list.selection.n_items == 1 }
    d.check('it is the Lamport entry') do
      win.call.entry_list.selection.model.get_item(0).citation_key == 'lamport1994latex'
    end
    d.shot('04-search')
  end

  d.step('clearing the search restores the list') do
    win.call.search_entry.text = ''
    d.check('both rows are listed again') { win.call.entry_list.selection.n_items == 2 }
  end

  d.step('saving writes the bibliography back') do
    win.call.save
  end

  d.step('the file on disk has the edit') do
    File.read(path).then do |written|
      d.check('the new title was written') { written.include?('Literate Programming, Revisited') }
      d.check('both entries were written') { written.scan(/^@/).length == 2 }
      d.check('the bibliography is no longer modified') { !win.call.bib.modified? }
    end
  end

  d.step('deleting a citation offers an undo') do
    win.call.delete_entry(win.call.bib.item(0))
  end

  d.step('the citation is gone and the toast is up') do
    d.check('one entry is left') { win.call.bib.n_items == 1 }
    d.check('the content pane was hidden') { !win.call.nav_split_view.show_content? }
    d.shot('05-deleted')
  end

  d.step('undoing the deletion puts it back') do
    win.call.widget.activate_action('undo_delete', nil)
  end

  d.step('the citation came back where it was') do
    d.check('two entries again') { win.call.bib.n_items == 2 }
    d.check('it went back to the first position') do
      win.call.bib.item(0).citation_key == 'knuth1984literate'
    end
  end

  d.step('copying the BibTeX for the selected citation') do
    win.call.entry_list.set_selected(1)
    win.call.widget.activate_action('copy_entry_bibtex', nil)
    d.check('the entry serialises as BibTeX') do
      win.call.bib.item(1).serialize.start_with?('@book{lamport1994latex,')
    end
  end

  d.step('the book shows notes rather than an abstract') do
    page = win.call.entry_page
    d.check('notes are visible for a book') { page.notes_preferences_group.visible? }
    d.check('the abstract is hidden for a book') { !page.abstract_preferences_group.visible? }
    d.check('the sidebar row shows the texered title') do
      page.title_entry.text.start_with?('\\LaTeX')
    end
    d.shot('06-book')
  end

  d.step('collapsing the split view hides the selection') do
    win.call.nav_split_view.collapsed = true
  end

  d.step('the sidebar selection is hidden while collapsed') do
    d.check('the selection is hidden') { win.call.entry_list.selection.hide_selection? }
    d.shot('07-collapsed')
    win.call.nav_split_view.collapsed = false
  end

  d.step('the new entry dialog opens') do
    win.call.widget.activate_action('new_entry', nil)
  end

  d.step('the new entry dialog is up and gated') do
    win.call.widget.visible_dialog.then do |dialog|
      d.check('a dialog is showing') { !dialog.nil? }
      d.check('it is titled Add new Entry') { dialog&.title == 'Add new Entry' }
      d.shot('08-new-entry', dialog)
      dialog&.close
    end
  end

  d.step('the about dialog opens') do
    win.call.widget.activate_action('about', nil)
  end

  d.step('the about dialog carries the metainfo') do
    win.call.widget.visible_dialog.then do |dialog|
      d.check('a dialog is showing') { !dialog.nil? }
      d.check('it names the application') { dialog&.application_name == 'Citations' }
      d.check('it carries the summary') { dialog&.comments == 'Manage your bibliography' }
      d.check('it carries the version') { dialog&.version == '0.11.0' }
      d.shot('09-about', dialog)
      dialog&.close
    end
  end

  d.step('the shortcuts dialog opens') do
    application.app.activate_action('shortcuts', nil)
  end

  d.step('the shortcuts dialog lists the sections') do
    win.call.widget.visible_dialog.then do |dialog|
      d.check('a dialog is showing') { !dialog.nil? }
      d.shot('10-shortcuts', dialog)
      dialog&.close
    end
  end

  d.step('adding an entry through the window') do
    win.call.new_entry("@misc{newkey,\n  title = {A New One},\n}")
  end

  d.step('the new entry is in the bibliography and selected') do
    d.check('three entries now') { win.call.bib.n_items == 3 }
    d.check('the new key is there') { win.call.bib.key_exists?('newkey') }
    d.check('the editor shows it') { win.call.entry_page.entry&.citation_key == 'newkey' }
    d.shot('11-new-entry-added')
  end

  d.step('a duplicate key is refused') do
    win.call.new_entry("@misc{newkey,\n  title = {Another},\n}")
    d.check('still three entries') { win.call.bib.n_items == 3 }
  end

  d.step('the recent files screen lists the file') do
    win.call.main_stack.visible_child = win.call.recent_files_stack
    d.check('the recent files page is showing') do
      win.call.recent_files_stack.visible_child == win.call.recent_files_stack_page
    end
    d.check('a row was built for the file') do
      !win.call.recent_files_list.get_row_at_index(0).nil?
    end
    d.check('the row is titled after the file') do
      win.call.recent_files_list.get_row_at_index(0).title == 'library.bib'
    end
  end

  d.step('the recent files screen renders') do
    d.shot('12-recent-files')
  end

  d.step('a tall window shows the whole editor') do
    win.call.main_stack.visible_child = win.call.nav_split_view
    win.call.widget.set_default_size(1100, 1400)
    win.call.entry_list.set_selected(0)
  end

  d.step('the editor shows the extras, the preview and the PDF row') do
    page = win.call.entry_page
    d.check('the extras repeater built a row per field') do
      page.extras_list_box.get_row_at_index(0)&.title == 'Journal'
    end
    d.check('the eprint row is the last extra of an article') do
      page.extras_list_box.get_row_at_index(7)&.title == 'ePrint'
    end
    d.check('the PDF stack offers to add one') do
      page.pdf_stack.visible_child == page.pdf_add_list_box
    end
    d.check('the citation preview shows the MLA rendering') do
      page.citation_preview.source_view.buffer.text.include?('The Computer Journal')
    end
    d.shot('13-editor-full')
    d.shot('13b-editor-box', page.editor_box)
  end

  d.step('the add pdf dialog opens from the empty PDF row') do
    win.call.entry_page.widget.activate_action('entry-page.set-pdf', nil)
  end

  d.step('the add pdf dialog is up, titled and empty') do
    win.call.entry_page.pdf_dialog.then do |pdf_dialog|
      d.check('the dialog object exists') { !pdf_dialog.nil? }
      d.check('it is titled Add PDF') { pdf_dialog&.dialog&.title == 'Add PDF' }
      d.check('the URL row starts empty') { pdf_dialog&.pdf_url_entry_row&.text == '' }
      d.check('no download is in flight yet') { !pdf_dialog&.progress_bar&.visible? }
      d.shot('13c-add-pdf-dialog', pdf_dialog&.dialog)
    end
  end

  d.step('typing a non-PDF URL into the add pdf dialog') do
    win.call.entry_page.pdf_dialog.pdf_url_entry_row.text = 'not-a-url'
  end

  d.step('attempting the download rejects the bad URL without a network call') do
    win.call.entry_page.pdf_dialog.then do |pdf_dialog|
      pdf_dialog.download_button.clicked
      d.check('the entry row took the error style') do
        pdf_dialog.pdf_url_entry_row.has_css_class?('error')
      end
      d.check('the dialog did not close itself') { !win.call.entry_page.pdf_dialog.nil? }
    end
  end

  d.step('cancelling the add pdf dialog') do
    win.call.entry_page.pdf_dialog.dialog.close
  end

  d.step('the entry is left without a PDF after cancelling') do
    page = win.call.entry_page
    d.check('the dialog is gone') { page.pdf_dialog.nil? }
    d.check('the PDF stack still offers to add one') do
      page.pdf_stack.visible_child == page.pdf_add_list_box
    end
    d.check('the citation still has no PDF on disk') { !Utils.has_pdf?(page.entry.citation_key) }
  end

  d.step('switching the citation template changes the rendering') do
    win.call.entry_page.citation_preview.template_drop_down.selected = 1
  end

  d.step('the AMS template rendered') do
    preview = win.call.entry_page.citation_preview
    d.check('the template kind was stored') { win.call.settings['template-kind'] == 1 }
    d.check('the AMS rendering is showing') do
      preview.source_view.buffer.text.include?('no. 2')
    end
    d.shot('14-ams-template')
  end

  d.step('the custom template reveals its editor') do
    win.call.entry_page.citation_preview.template_drop_down.selected = 6
  end

  d.step('a file handed to the application opens in its own window') do
    application.open_files([Gio::File.new_for_path(path)])
  end

  d.step('the second window opened the bibliography') do
    application.windows.last.then do |second|
      d.check('a second window was built') { application.windows.length == 2 }
      d.check('it is not the first one') { !second.equal?(win.call) }
      d.check('it loaded the two saved citations') { second.bib&.n_items == 2 }
      d.check('it is showing the split view') do
        second.main_stack.visible_child == second.nav_split_view
      end
      d.shot('16-opened-from-argv', second.widget)
      second.widget.close
    end
  end

  d.step('the custom template editor is revealed') do
    preview = win.call.entry_page.citation_preview
    d.check('the revealer opened') { preview.template_revealer.reveal_child? }
    d.check('it holds the stored custom article template') do
      preview.text_view.buffer.text == win.call.settings['custom-article-template']
    end
    d.shot('15-custom-template')
  end
end
