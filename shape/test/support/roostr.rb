# The commands of this project, each run in the test's workspace.
#
#   overview               roostr with no arguments
#   show "book-workshop"   roostr show <id>; `show` alone runs it without an id
#   validate               roostr validate
#   scan                   roostr scan
#   init                   roostr init
#   changes                roostr changes; changes "--since", "3d" passes arguments on
#
# The overview and changes read the date. Their clock is always pinned, so a test never changes
# its answer as real time passes: today is 2026-10-10 unless the test says today: "2026-11-02",
# and a day starts at 00:00 UTC on every machine.
#
#   atelier_under_git                      the Atelier fixture as a Git repository with one commit
#   atelier_after_the_waiting_list_week    ...and a week of commits on top
#
# roostr scan runs the project's own script, .roostr/scan. In a test that script is a fake:
#
#   scan_prints "# covers: http\n"                      a script that prints this and exits 0
#   scan_prints text, stderr: "no database", exit: 3    ...and writes to stderr, and exits 3
#   scan_script "leaves-a-trace.sh"                     test/fixtures/scans/leaves-a-trace.sh
module Roostr
  TODAY = "2026-10-10"

  def overview(today: TODAY)       = cli(env: clock(today))
  def changes(*args, today: TODAY) = cli("changes", *args, env: clock(today))
  def show(*id)  = cli("show", *id)
  def validate   = cli("validate")
  def scan       = cli("scan")
  def init       = cli("init")

  def scan_prints(output, stderr: nil, exit: 0)
    script = ["#!/bin/sh"]
    script += ["cat <<'END_OF_OUTPUT'", output.chomp, "END_OF_OUTPUT"] unless output.empty?
    script += ["cat >&2 <<'END_OF_ERRORS'", stderr, "END_OF_ERRORS"] if stderr
    write_executable ".roostr/scan", [*script, "exit #{exit}"].join("\n") + "\n"

    printed = output.empty? ? "  (nothing)" : output.chomp.gsub("\t", "→").gsub(/^/, "  ")
    note_situation ".roostr/scan is an executable that #{"writes #{stderr.inspect} to stderr, " if stderr}" \
                   "exits #{exit} and prints this to stdout (→ is a tab character):\n#{printed}"
  end

  # The comment under the script's first line says what it does, and is shown when a test fails.
  def scan_script(name)
    source = fixture("scans/#{name}")
    write_executable ".roostr/scan", source
    note_situation ".roostr/scan is an executable that #{source.lines.drop(1).take_while { |line| line.start_with?("#") }.map { |line| line.delete_prefix("#").strip }.join(" ")}"
  end

  # Atelier as a Git repository with one commit, dated 2026-01-15. A failure then says which
  # process lists which file, which the transcript of a command cannot show.
  def atelier_under_git
    workspace "atelier", git: true
    note_situation "In this project book-workshop lists app/models/booking.rb and app/services/book_workshop.rb, " \
                   "cancel-booking lists app/models/booking.rb, and remind-attendees lists app/jobs/booking_reminder_job.rb."
  end

  # Atelier, committed on 2026-01-15, then six commits. With today on 2026-10-10 the last 7 days
  # start on 2026-10-03, so the first of the six is outside them. Nothing is left uncommitted.
  def atelier_after_the_waiting_list_week
    atelier_under_git

    edit_file "app/services/book_workshop.rb"
    commit "Round prices to whole euros", on: "2026-09-24"

    edit_file "app/models/booking.rb"
    write_file "app/controllers/waiting_list_controller.rb", "# Placeholder standing in for app/controllers/waiting_list_controller.rb in the Atelier fixture.\n"
    write_file "app/jobs/waiting_list_job.rb", "# Placeholder standing in for app/jobs/waiting_list_job.rb in the Atelier fixture.\n"
    write_file ".roostr/entrypoints.tsv", fixture("entrypoints/waiting-list-added.tsv")
    commit "Add the waiting list", on: "2026-10-05"

    edit_file "app/controllers/bookings_controller.rb"
    write_file ".roostr/entrypoints.tsv", fixture("entrypoints/booking-page-dropped.tsv")
    commit "Drop the booking page", on: "2026-10-06"

    edit_file "app/models/booking.rb"
    edit_file ".roostr/processes/cancel-booking.yml"
    commit "Offer a freed place to the next person waiting", on: "2026-10-07"

    edit_file "app/jobs/booking_reminder_job.rb"
    commit "Send the reminder two days ahead", on: "2026-10-08"

    edit_file "app/controllers/workshops_controller.rb"
    commit "Show the trainer on the workshop page", on: "2026-10-09"
  end

  private

  def clock(today) = { "ROOSTR_TODAY" => today, "TZ" => "UTC" }
end
