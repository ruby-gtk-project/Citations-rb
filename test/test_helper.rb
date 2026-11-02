# frozen_string_literal: true

# A dependency-free check harness for the port's non-widget code.
#
# The dev shell has no minitest, and the project's spec_helper pulls in gems
# that are not in it either, so the model tests run as plain Ruby scripts:
# `ruby test/test_texer.rb`. Each test is named after the upstream #[test]
# function it is ported from, so the two suites can be compared row by row.
module TestHelper
  @failures = []
  @checks = 0

  class << self
    attr_reader :failures, :checks

    def test(name)
      @current = name
      yield
    end

    def assert_equal(expected, actual, context = nil)
      @checks += 1

      if expected == actual
        nil
      else
        @failures << "#{@current}#{context.nil? ? '' : " (#{context})"}:\n" \
                     "    expected #{expected.inspect}\n" \
                     "    actual   #{actual.inspect}"
      end
    end

    def assert(actual, context = nil) = assert_equal(true, !!actual, context)

    def report(suite)
      puts "#{@checks - @failures.length} of #{@checks} checks passed in #{suite}"
      @failures.each { |failure| puts "  FAIL #{failure}" }

      if @failures.empty?
        puts "#{suite} OK"
      else
        puts "#{suite} FAILED (#{@failures.length})"
      end

      exit(@failures.empty? ? 0 : 1)
    end
  end
end

$LOAD_PATH.unshift(File.expand_path('../lib', __dir__))
