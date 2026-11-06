# Running the program, or any other executable in the work folder. The kit is for
# command-line programs: a command, in a folder, with files.
#
#   cli "redeem", "GIFT-2041"        the program (PROGRAM in test_helper.rb)
#   script "bin/backup", "data"      any executable in the work folder
#
# Each one runs in the test's workspace and remembers its result for the
# assertions below. Options: stdin: "text", env: { "NAME" => "value" },
# from: "app/models" (runs it in that subfolder of the workspace),
# stdout: :closed (stdout is a pipe whose reader has already gone, as in `| head -c1` once
# head has exited, so the first write to it fails; nothing can be read from it),
# stdin_held_open: true (stdin: is written, and stdin then stays open until the command has
# exited, as from a caller that never closes it), stdout: :full (stdout is /dev/full, a full
# disk: every write fails with No space left on device; the test is skipped where there is
# none, as on macOS), from_a_deleted_folder: true (it runs in app/drafts, which the shell that
# starts it has deleted after going into it).
#
#   environment_for_commands "PATH" => "..."   for every command after it in the test; a
#                                              command's own env: wins. Set by helpers, which
#                                              say under a failure what they set
#
# For a program that prints one JSON object on stdout:
#
#   assert_json "format", 1                       the value at a path of keys, and of positions in lists
#   assert_json "process.files.2.note", nil       ...positions count from 1. A key that is not there fails,
#                                                 so null and absent stay apart
#   assert_json "processes.*.id", ["a", "b"]      * is every item of a list
#   assert_json "problems.1.message", /refunds/   a pattern for a string; String for any string
#   assert_json_is expected("show.json")          the whole object equals the file, both parsed, so
#                                                 spacing and the order of keys do not matter
#   assert_json_is expected("p.json"), at: "project"   ...or the value at a path
#
# A failure transcript shows stdout indented when it is one JSON object.
#
#   assert_stderr_is ""                           the whole of stderr: here, nothing at all
#   assert_finished_within 3                      the program exited within 3 s of starting; for when
#                                                 returning promptly is the behaviour. Whatever it left
#                                                 running is stopped once it has exited
require "open3"
require "json"
require "stringio"

module Commands
  COMMAND_TIMEOUT = 20 # seconds

  Finished = Struct.new(:line, :stdout, :stderr, :exit, :situation, :seconds) do
    def to_s
      ["  $ #{line}", "  exit #{exit}", stream("stdout", stdout), stream("stderr", stderr), *before].join("\n")
    end

    # What a helper put in place before the command ran and the transcript alone would not show.
    def before
      return [] if situation.empty?
      ["", "  Before the command:", *situation.map { |note| note.gsub(/^/, "    ") }]
    end

    def stream(name, text)
      return "  #{name}: (empty)" if text.empty?
      object = Commands.json_object(text) if name == "stdout"
      return "  #{name}, one JSON object, shown indented:\n#{lines(JSON.pretty_generate(object), 80)}" if object
      "  #{name}:\n#{lines(text, 40)}"
    end

    def lines(text, most)
      shown = text.lines.first(most).join
      shown += "\n" unless shown.end_with?("\n")
      shown += "    … #{text.lines.size - most} more lines\n" if text.lines.size > most
      shown.gsub(/^/, "    ").chomp
    end
  end

  # The text read as JSON, when it is exactly one object; otherwise nil.
  def self.json_object(text)
    value = JSON.parse(text)
    value if value.is_a?(Hash)
  rescue JSON::ParserError
    nil
  end

  def cli(*args, **options)
    execute Target.path(PROGRAM), args, shown_as: File.basename(PROGRAM), **options
  end

  def script(path, *args, **options)
    execute Target.path(path), args, shown_as: path, **options
  end

  def stdout = finished.stdout
  def stderr = finished.stderr

  def assert_success = assert_exit(0)

  def assert_failure
    assert finished.exit != 0, "Expected the command to fail, but it exited 0.\n\n#{finished}"
  end

  def assert_exit(code)
    assert finished.exit == code, "Expected exit #{code}, got #{finished.exit}.\n\n#{finished}"
  end

  def assert_stdout(text) = assert_stream("stdout", stdout, text)
  def assert_stderr(text) = assert_stream("stderr", stderr, text)
  def refute_stdout(text) = refute_stream("stdout", stdout, text)
  def refute_stderr(text) = refute_stream("stderr", stderr, text)

  # The whole of stdout, character for character. For output whose layout is the feature.
  def assert_stdout_is(text)
    assert_same_text text, stdout, "The output of `#{finished.line}`", after: "\n\n#{finished}"
  end

  # The whole of stderr, character for character.
  def assert_stderr_is(text)
    assert_same_text text, stderr, "What `#{finished.line}` wrote to stderr", after: "\n\n#{finished}"
  end

  # Environment variables for every command run after this in the test.
  def environment_for_commands(variables)
    environment.merge!(variables.transform_values(&:to_s))
  end

  # Said by a helper about what it set up; shown under the transcript when a test fails.
  def note_situation(text)
    situation << text
  end

  # Every line of stdout fits in the given number of columns. For plain-text screens.
  def assert_lines_within(columns)
    long = stdout.lines.map(&:chomp).select { |line| line.length > columns }
    shown = long.map { |line| "  #{line.length} columns: #{line}" }.join("\n")
    assert long.empty?, "Expected every line of stdout to fit in #{columns} columns. These do not:\n\n#{shown}\n\n#{finished}"
  end

  # The program exited within this many seconds of starting. Only where returning promptly is
  # the behaviour, as when something it started keeps running.
  def assert_finished_within(seconds)
    assert finished.seconds < seconds,
           "Expected the command to finish within #{seconds} s; it exited after #{format("%.1f", finished.seconds)} s.\n\n#{finished}"
  end

  # stdout has exactly this many lines.
  def assert_stdout_lines(count)
    assert_equal count, stdout.lines.size, "Expected stdout to be #{count} line(s), got #{stdout.lines.size}.\n\n#{finished}"
  end

  # The value at a path in the JSON object on stdout: keys and positions, joined by dots.
  def assert_json(path, expected)
    actual = json_at(path)
    assert expected === actual, "Expected #{path} in the JSON on stdout to be:#{shown_json(expected)}\nIt is:#{shown_json(actual)}\n\n#{finished}"
  end

  # The JSON object on stdout, or the value at a path in it, equals the expected JSON text.
  def assert_json_is(expected_text, at: nil)
    expected = JSON.parse(expected_text)
    actual = at ? json_at(at) : json
    return assert(true) if expected == actual
    assert_same_text "#{JSON.pretty_generate(expected)}\n", "#{JSON.pretty_generate(in_order_of(expected, actual))}\n",
                     "The JSON on stdout#{" at #{at}" if at} (both shown indented, keys in the expected order; " \
                     "the spacing and key order of the output do not matter)", after: "\n\n#{finished}"
  end

  private

  def json
    parsed = Commands.json_object(stdout)
    return parsed if parsed
    reason = if stdout.strip.empty? then "stdout is empty"
             elsif (JSON.parse(stdout) rescue nil).nil? then "it does not parse as JSON, or there is more than one value"
             else "it is a JSON value that is not an object"
             end
    flunk "Expected stdout to be exactly one JSON object and nothing else, but #{reason}.\n\n#{finished}"
  end

  def json_at(path)
    dig_json(json, path.split("."), [], path)
  end

  def dig_json(value, steps, done, path)
    return value if steps.empty?
    step, *rest = steps
    here = done.empty? ? "the object" : done.join(".")
    if step == "*"
      missing_json(path, here, "is not a list", value) unless value.is_a?(Array)
      return value.map.with_index(1) { |item, position| dig_json(item, rest, done + [position.to_s], path) }
    end
    case value
    when Hash
      missing_json(path, here, "has no key #{step.inspect}", value) unless value.key?(step)
      dig_json(value[step], rest, done + [step], path)
    when Array
      position = step.match?(/\A[1-9]\d*\z/) && step.to_i
      missing_json(path, here, "is a list, and #{step} is not a position in it (positions count from 1)", value) unless position
      missing_json(path, here, "is a list of #{value.size} item(s), so it has no item #{position}", value) if position > value.size
      dig_json(value[position - 1], rest, done + [step], path)
    else
      missing_json(path, here, "is neither an object nor a list", value)
    end
  end

  def missing_json(path, here, reason, value)
    flunk "Expected the JSON on stdout to have #{path}, but #{here} #{reason}. #{here.sub(/\Athe/, "The")} is:#{shown_json(value)}\n\n#{finished}"
  end

  # A value as JSON, on the same line when short, indented below when long.
  def shown_json(value)
    case value
    when Regexp then " a string matching #{value.inspect}"
    when Class then " any #{value.name.downcase}"
    else
      compact = JSON.generate(value)
      compact.length <= 60 ? " #{compact}" : "\n#{JSON.pretty_generate(value).gsub(/^/, "    ")}"
    end
  end

  # The actual value with its keys in the order the expected one has them, so that a diff
  # shows only what differs.
  def in_order_of(expected, actual)
    case [expected, actual]
    in [Hash, Hash]
      keys = (expected.keys & actual.keys) + (actual.keys - expected.keys)
      keys.to_h { |key| [key, in_order_of(expected[key], actual[key])] }
    in [Array, Array]
      actual.map.with_index { |item, index| in_order_of(expected[index], item) }
    else
      actual
    end
  end

  def situation = @situation ||= []
  def environment = @environment ||= {}

  def finished
    @finished or raise Minitest::Assertion, "No command has been run in this test yet."
  end

  def assert_stream(name, actual, text)
    assert text === actual || actual.include?(text.to_s), "Expected #{name} to include #{text.inspect}.\n\n#{finished}"
  end

  def refute_stream(name, actual, text)
    refute text === actual || actual.include?(text.to_s), "Expected #{name} not to include #{text.inspect}.\n\n#{finished}"
  end

  def execute(executable, args, shown_as:, stdin: "", env: {}, from: nil, stdout: :read, stdin_held_open: false, from_a_deleted_folder: false)
    skip "This system has no /dev/full (macOS has none), which the test needs as a stdout that cannot be written." if stdout == :full && !File.exist?("/dev/full")
    args = args.map(&:to_s)
    line = [shown_as, *args.map { |arg| arg.to_s.match?(/\A[\w\-.\/:=@,+]+\z/) ? arg.to_s : arg.to_s.inspect }].join(" ")
    line = "#{env.map { |name, value| "#{name}=#{value.to_s.match?(/\A[\w\-.\/:=@,+]+\z/) ? value : value.to_s.inspect}" }.join(" ")} #{line}" unless env.empty?
    line = "cd #{from} && #{line}" if from
    line = "#{line}   (its stdout is a pipe whose reader has already gone)" if stdout == :closed
    line = "#{line}   (its stdin is written and then kept open: no end of input)" if stdin_held_open
    line = "#{line} >/dev/full   (its stdout is a full disk: every write fails with No space left on device)" if stdout == :full
    if from_a_deleted_folder
      line = "cd app/drafts && rmdir \"$PWD\" && #{line}   (run in a folder deleted after the shell went into it)"
      FileUtils.mkdir_p(File.join(dir, "app/drafts"))
      args = ["-c", 'cd "$1" && rmdir "$1" && shift && exec "$@"', "sh", File.join(dir, "app/drafts"), executable, *args]
      executable = "/bin/sh"
    end
    folder = from ? File.join(dir, from) : dir
    raise ArgumentError, "from: #{from} is not a folder in the test's workspace" unless File.directory?(folder)

    unless File.executable?(executable)
      raise Minitest::Assertion, "The tests run #{executable.delete_prefix("#{Target.root}/")} in the work folder, and nothing executable is there.\n\n  $ #{line}"
    end

    # The kit's own git status may refresh the index, so the files are remembered after it.
    before = [*situation, *repository_state]
    remember_workspace
    # Git inside the program behaves the same on every machine: fixed identity, no user or system config.
    # Its own process group, so that a timeout stops everything the program started.
    variables = Workspace::GIT_ENV.merge(environment, env.transform_values(&:to_s))
    started_at = Process.clock_gettime(Process::CLOCK_MONOTONIC)
    started = case stdout
              when :closed then start_with_stdout_closed(variables, executable, args, folder)
              when :full then start_with_stdout_full(variables, executable, args, folder)
              else Open3.popen3(variables, executable, *args, chdir: folder, pgroup: true)
              end
    wait_for(*started, line: line, stdin: stdin, before: before, started_at: started_at, stdin_held_open: stdin_held_open)
  end

  # A pipe whose reading end is closed before the program starts: its first write to stdout
  # fails, as when the reader of a pipeline has exited. Nothing is read from it.
  def start_with_stdout_closed(variables, executable, args, folder)
    gone, stdout = IO.pipe
    gone.close
    stdin, input = IO.pipe
    error, stderr = IO.pipe
    pid = Process.spawn(variables, executable, *args, chdir: folder, pgroup: true, in: stdin, out: stdout, err: stderr)
    [stdin, stdout, stderr].each(&:close)
    [input, StringIO.new, error, Process.detach(pid)]
  end

  # stdout is /dev/full: every write to it fails with ENOSPC. Nothing is read from it.
  def start_with_stdout_full(variables, executable, args, folder)
    full = File.open("/dev/full", "w")
    stdin, input = IO.pipe
    error, stderr = IO.pipe
    pid = Process.spawn(variables, executable, *args, chdir: folder, pgroup: true, in: stdin, out: full, err: stderr)
    [stdin, full, stderr].each(&:close)
    [input, StringIO.new, error, Process.detach(pid)]
  end

  def wait_for(input, output, error, waiter, line:, stdin:, before:, started_at:, stdin_held_open: false)
    begin
      input.write(stdin)
      input.flush
    rescue Errno::EPIPE
      # The program stopped reading its input; its exit code and output say why.
    end
    input.close unless stdin_held_open
    out = Thread.new { output.read.scrub }
    err = Thread.new { error.read.scrub }
    [out, err].each { |reader| reader.report_on_exception = false }
    unless waiter.join(COMMAND_TIMEOUT)
      begin
        Process.kill("KILL", -waiter.pid) # the whole process group, see pgroup above
      rescue Errno::ESRCH
        # It ended in the moment between the timeout and the kill.
      end
      waiter.join
      raise Minitest::Assertion, "The command was still running after #{COMMAND_TIMEOUT} s and was stopped.\n\n  $ #{line}"
    end
    seconds = Process.clock_gettime(Process::CLOCK_MONOTONIC) - started_at
    # A child the program left running (a helper its scan started) may still hold the pipes;
    # the program's answer is complete once it has exited, so the leftovers are stopped.
    begin
      Process.kill("KILL", -waiter.pid)
    rescue Errno::ESRCH, Errno::EPERM
      # Nothing of it is left.
    end
    status = waiter.value
    @finished = Finished.new(line, out.value, err.value, status.exitstatus || "none (killed by signal #{status.termsig})", before, seconds)
  ensure
    [input, output, error].each { |stream| stream.close unless stream.closed? }
  end
end
