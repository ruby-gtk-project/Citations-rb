# frozen_string_literal: true

require_relative "../main"

module Citations
  # Citations::Application is the name exe/citations launches. The port keeps
  # one file per upstream source, so the application itself is App, ported
  # from upstream's src/application.rs and src/main.rs into lib/main.rb.
  Application = App
end
