class CheckProcessesTest < AcceptanceTest
  def setup = atelier_under_git # committed once, no scan script; booking.rb is listed by book-workshop and cancel-booking, book_workshop.rb by book-workshop, booking_reminder_job.rb by remind-attendees

  test "without --base, a listed file committed after its record is reported under PROCESSES by process and file, with the remedy naming the record and checked: and today's date, exit 1" do
    edit_file "app/jobs/booking_reminder_job.rb"
    commit "Send the reminder two days ahead", on: "2026-10-06"

    check

    assert_exit 1
    assert_stdout "PROCESSES\n"
    assert_stdout "\n    remind-attendees\n      app/jobs/booking_reminder_job.rb\n"
    assert_stdout ".roostr/processes/<id>.yml"
    assert_stdout "checked: 2026-10-10"
  end

  test "with --base HEAD, a listed file committed after its record before HEAD is old debt and is not reported when the work after HEAD did not touch it" do
    edit_file "app/jobs/booking_reminder_job.rb"
    commit "Send the reminder two days ahead", on: "2026-10-06"
    edit_file "app/controllers/workshops_controller.rb" # the work after HEAD; listed by no record

    check "--base", "HEAD"

    assert_success
    refute_stdout "PROCESSES"
  end

  test "with --base HEAD, a listed file with an edit that is not committed is reported under PROCESSES" do
    edit_file "app/jobs/booking_reminder_job.rb"

    check "--base", "HEAD"

    assert_exit 1
    assert_stdout "\n    remind-attendees\n      app/jobs/booking_reminder_job.rb\n"
  end

  test "with --base HEAD, writing a new checked date into the record, not committed, clears the process" do
    edit_file "app/jobs/booking_reminder_job.rb"
    write_file ".roostr/processes/remind-attendees.yml", fixture("processes/remind-attendees-checked.yml") # the same record with checked: 2026-10-08

    check "--base", "HEAD"

    assert_success
    assert_stdout_is "No problems found\n"
  end

  test "with --base main, a branch whose one commit changed a listed file and its record together passes" do
    branch "longer-reminders"
    edit_file "app/jobs/booking_reminder_job.rb"
    edit_file ".roostr/processes/remind-attendees.yml"
    commit "Send the reminder two days ahead", on: "2026-10-06"

    check "--base", "main"

    assert_success
    assert_stdout_is "No problems found\n"
  end

  test "with --base main, a record updated on the branch after its file, whose file changed again in a later commit, is reported" do
    branch "longer-reminders"
    edit_file "app/jobs/booking_reminder_job.rb"
    edit_file ".roostr/processes/remind-attendees.yml"
    commit "Send the reminder two days ahead", on: "2026-10-06"
    edit_file "app/jobs/booking_reminder_job.rb"
    commit "Send the reminder at 09:00", on: "2026-10-07"

    check "--base", "main"

    assert_exit 1
    assert_stdout "\n    remind-attendees\n      app/jobs/booking_reminder_job.rb\n"
  end

  test "with --base HEAD, a process with old debt on one listed file and a new edit to another lists only the file changed after HEAD" do
    edit_file "app/models/booking.rb"
    commit "Hold the place while the payment is pending", on: "2026-10-06"
    edit_file "app/services/book_workshop.rb"

    check "--base", "HEAD"

    assert_exit 1
    assert_stdout "\n    book-workshop\n      app/services/book_workshop.rb\n"
    refute_stdout "app/models/booking.rb"
  end

  test "a new process record, not committed yet, that lists files the agent has just written is not reported" do
    write_file "app/controllers/waiting_list_controller.rb", "class WaitingListController < ApplicationController\nend\n"
    write_file "app/jobs/waiting_list_job.rb", "class WaitingListJob < ApplicationJob\nend\n"
    write_file ".roostr/entrypoints.tsv", fixture("entrypoints/waiting-list-added.tsv") # rescanned: adds http POST /waiting_list and job WaitingListJob
    write_file ".roostr/processes/join-waiting-list.yml", fixture("processes/join-waiting-list.yml") # lists both new files and both new entrypoints

    check "--base", "HEAD"

    assert_success
    assert_stdout_is "No problems found\n"
  end

  test "with --base main on a branch, work committed on the branch and work not committed on top of it are judged in one run" do
    branch "longer-reminders"
    edit_file "app/jobs/booking_reminder_job.rb"
    commit "Send the reminder two days ahead", on: "2026-10-06"
    edit_file "app/services/book_workshop.rb"

    check "--base", "main"

    assert_exit 1
    assert_stdout "\n    book-workshop\n      app/services/book_workshop.rb\n"
    assert_stdout "\n    remind-attendees\n      app/jobs/booking_reminder_job.rb\n"
  end

  test "a listed file whose path has a space and an accented letter is reported under PROCESSES by its path as the record writes it" do
    write_file "app/gift cards/café.rb", "class GiftCard\nend\n"
    write_file ".roostr/processes/sell-gift-cards.yml", fixture("processes/sell-gift-cards.yml") # lists app/gift cards/café.rb
    commit "Sell gift cards", on: "2026-10-05"
    edit_file "app/gift cards/café.rb"

    check "--base", "HEAD"

    assert_exit 1
    assert_stdout "\n    sell-gift-cards\n      app/gift cards/café.rb\n"
  end
end
