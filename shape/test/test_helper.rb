require "minitest/autorun"

# Program output is read as UTF-8 whatever locale the machine happens to have.
Encoding.default_external = Encoding::UTF_8

# ── This project ─────────────────────────────────────────────────────────
PROGRAM = "bin/roostr" # the executable the tests run, relative to the work folder
# ─────────────────────────────────────────────────────────────────────────

TEST_ROOT = __dir__

Dir[File.join(TEST_ROOT, "support", "*.rb")].sort.each { |file| require file }

class AcceptanceTest < Minitest::Test
  include Fixtures, ExactText, Workspace, Commands, Roostr

  # test "a redeemed voucher cannot be redeemed again" do ... end
  def self.test(sentence, &block)
    name = "test_#{sentence.gsub(/\W+/, "_")}"
    raise ArgumentError, "#{self} already has a test named: #{sentence}" if method_defined?(name)
    define_method(name, &block)
  end
end
