class SessionMapTest < AcceptanceTest
  def setup = atelier_under_git

  test "on startup, session start prints one line on stdout: a JSON object whose one key, hookSpecificOutput, holds hookEventName SessionStart and the project map as additionalContext; nothing on stderr, exit 0" do
    session_start

    assert_success
    assert_map "roostr project map of Atelier"
    assert_stderr_is ""
  end

  test "Atelier's map: the header, Stack and Description, Watch out, Operations, then the processes by group, each as id: description [first files]" do
    session_start

    assert_map_is expected("session_map_atelier.txt")
  end

  { "clear" => "clear", "compact" => "compact", "no source" => nil,
    "restart, a source roostr does not know" => "restart", "the number 42 as the source" => 42 }.each do |said, source|
    test "a session that starts with #{said} gets the map" do
      session_start source: source

      assert_success
      assert_map "roostr project map of Atelier"
    end
  end

  ["resume", "fork"].each do |source|
    test "a session that starts with #{source} gets no map, since its history holds the earlier one: nothing on stdout, exit 0" do
      session_start source: source

      assert_success
      assert_stdout_is ""
    end
  end

  test "a session started by fork, though it gets no map, is recorded: a change committed after it is listed by check --session" do
    session_start source: "fork"
    reminder_job_changed_and_committed

    check_session

    assert_exit 1
    assert_stdout "\n    remind-attendees\n      app/jobs/booking_reminder_job.rb\n"
  end

  test "a description with a double quote, a backslash, an accented letter and an emoji is in the map as written, the accent and the emoji as UTF-8 in the JSON" do
    records_added "print-day-sheet"

    session_start

    assert_map "\nWorkshops:\n- print-day-sheet: Print the \"day sheet\" for the kiln room at C:\\Studio, café included 🏺 [app/controllers/workshops_controller.rb]"
    assert_stdout "café included 🏺"
  end

  test "the operation named deploy and the environments are not in the map" do
    write_file ".roostr/project.yml", fixture("project/operations-of-every-kind.yml") # deploy with a note, staging and production on Hetzner

    session_start

    refute_map "kamal"
    refute_map "Hetzner"
  end

  test "operations that ship are not in the map: any whose name has the word deploy, release or publish in any letter case, once the characters that do not show are dropped (deploy-production, deploy:staging, staging.deploy, Release, publish_gem, de<soft hyphen>ploy); deployment-status and predeploy-check stay" do
    write_file ".roostr/project.yml", fixture("project/operations-that-ship.yml")

    session_start

    assert_map "\n\nOperations:\n- start: `bin/dev`\n- deployment-status: `bin/kamal app details`\n- predeploy-check: `bin/rails test:system`\n- test: `bin/rails test`\n\n"
  end

  test "a named pipe among the records is not opened, so it does not hold up session start: the map is Atelier's, without it" do
    named_pipe ".roostr/processes/refund-booking.yml"

    session_start

    assert_map_is expected("session_map_atelier.txt")
  end

  test "an operation with a note is listed as - name: `command` (note)" do
    write_file ".roostr/project.yml", fixture("project/operations-of-every-kind.yml") # seed is bin/rails db:seed, noted

    session_start

    assert_map "\n- seed: `bin/rails db:seed` (empties the bookings first)\n"
  end

  test "an operation whose command has several lines shows its first line, then ..." do
    write_file ".roostr/project.yml", fixture("project/operations-of-every-kind.yml") # setup is bin/setup, then bin/rails db:seed

    session_start

    assert_map "\nOperations:\n- setup: `bin/setup` ...\n- start: `bin/dev`\n"
  end

  test "an operation with no command is left out" do
    write_file ".roostr/project.yml", fixture("project/operations-of-every-kind.yml") # reset: with nothing after it

    session_start

    refute_map "reset"
  end

  test "groups are in alphabetical order, each process under its group by id, and the processes without a group last, under other" do
    records_added "sync-calendar", "export-bookings", "import-workshops" # groups Imports and Reports, and no group

    session_start

    assert_map(/\nBookings:\n- book-workshop: .*\n- cancel-booking: .*\nImports:\n- sync-calendar: .*\nNotifications:\n- remind-attendees: .*\nReports:\n- export-bookings: .*\nother:\n- import-workshops: .*\z/)
  end

  test "a process lists its first two files in brackets, and one with no files has no brackets" do
    records_added "move-booking", "import-workshops" # three files, and none

    session_start

    assert_map "\n- move-booking: Move a booking to a later workshop with a free place. [app/models/booking.rb, app/services/move_booking.rb]\n"
    assert_map(/\n- import-workshops: Import workshops from the studio calendar\z/)
  end

  test "a record that cannot be read is left out of the map without a word, and the others are listed" do
    write_file ".roostr/processes/refund-booking.yml", "name: Refund a booking\ngroup: Bookings\n" # no description

    session_start

    refute_map "refund"
    assert_map "\n- cancel-booking: "
  end

  test "no entrypoint is in the map, neither one a record lists nor one of entrypoints.tsv, nor an area no process names, Workshops" do
    session_start

    refute_map "BookingReminderJob"
    refute_map "GET /workshops"
    refute_map "Workshops:"
  end

  test "what the overview says about recent work is not in the map: no commit message, no LAST 7 DAYS, no NEEDS ATTENTION, no out of date" do
    reminder_job_changed_and_committed # on 2026-10-10, "Send the reminder two days ahead"; remind-attendees.yml not updated

    session_start

    assert_map "roostr project map of Atelier"
    refute_map "two days ahead"
    refute_map(/LAST 7 DAYS|NEEDS ATTENTION|out of date/i)
  end

  test "session start never runs the scan script" do
    scan_script "leaves-a-trace.sh"

    session_start

    assert_map "roostr project map of Atelier"
    refute_file "scan-was-run"
  end

  test "the hook's input written and stdin then kept open, as by a runner that never closes it: session start still records the session, prints the map and exits within 5 s" do
    session_start stdin_held_open: true

    assert_finished_within 5
    assert_written_only_under ".git/roostr/sessions/"
    assert_map "roostr project map of Atelier"
  end
end
