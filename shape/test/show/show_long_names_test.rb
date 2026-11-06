class ShowLongNamesTest < AcceptanceTest
  def setup = workspace("billing-service") # send-payment-reminder-to-overdue-clients-by-email lists an entrypoint of 87 characters, in a file whose path has 81, and has an outcome named with 48

  test "an entrypoint longer than the line is printed whole on its entry line, kind and name together" do
    show "send-payment-reminder-to-overdue-clients-by-email"

    assert_success
    assert_stdout "  entry    http POST /api/v1/organisations/{organisationId}/invoices/{invoiceId}/payment-reminders\n"
  end

  test "a path longer than the line is printed whole, never broken, both under the entrypoint and under code" do
    show "send-payment-reminder-to-overdue-clients-by-email"

    assert_stdout "/payment-reminders\n           src/main/java/com/example/billing/reminders/OverdueInvoiceReminderController.java\n"
    assert_stdout "  code     src/main/java/com/example/billing/reminders/OverdueInvoiceReminderController.java\n"
  end

  test "an outcome name longer than 36 characters is printed whole on a line of its own, and its condition and result start on the next line, in the outcome column" do
    show "send-payment-reminder-to-overdue-clients-by-email"

    assert_stdout "OUTCOMES\n" \
                  "  reminder-mailed-and-noted-in-the-invoice-history\n" \
                  "                                         the invoice is overdue and unpaid: a\n"
    assert_stdout "  skipped                                the client paid in the meantime"
  end
end
