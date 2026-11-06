class LongNamesTest < AcceptanceTest
  def setup = workspace("billing-service") # ids of 49 and 48 characters beside refund-invoice; an area of 42 characters, a group of 53, an operation of 56, an environment of 40

  test "an id longer than 36 characters is printed whole on a line of its own, and its description starts on the next line, in the description column" do
    overview

    assert_success
    assert_stdout "    send-payment-reminder-to-overdue-clients-by-email\n" \
                  "                                           Mail each client whose invoice is\n" \
                  "                                           overdue, once a week.\n"
  end

  test "an id that fits keeps its place: with longer ids shown, the id column is 36 characters wide plus three spaces, and refund-invoice's description starts there" do
    overview

    assert_stdout "    refund-invoice                         Refund an invoice to the card it was\n" \
                  "                                           paid with.\n"
  end

  test "two ids that share their first 44 characters are both printed whole, so they differ on the screen and either can be copied into roostr show" do
    overview

    assert_stdout "    send-payment-reminder-to-overdue-clients-by-email\n"
    assert_stdout "    send-payment-reminder-to-overdue-clients-by-post\n"
    refute_stdout "..."
  end

  test "an area name longer than 36 characters is printed whole on a line of its own, and its counts start on the next line, in the counts column" do
    overview

    assert_stdout "  ReminderSchedulingAndDunningAdministration\n" \
                  "                                         http 2\n"
    assert_stdout "  Billing                                http 2\n"
  end

  test "a group longer than 36 characters that matches no area is printed whole as its heading, with its process under it" do
    overview

    assert_stdout "  QuarterlyReturnsAndAnnualStatementsForTheTaxAuthority\n" \
                  "    file-quarterly-vat-return "
  end

  test "an operation name longer than 36 characters is printed whole on a line of its own, and its command starts on the next line, in the command column" do
    overview

    assert_stdout "  start                                   ./gradlew bootRun\n" \
                  "  regenerate-the-api-client-from-the-openapi-specification\n" \
                  "                                          ./gradlew openApiGenerate"
  end

  test "a command longer than the line is printed whole on one line, never wrapped: a command is copied, and a wrapped one pastes as two" do
    overview

    assert_stdout "                                          ./gradlew openApiGenerate --input src/main/resources/openapi.yaml --output build/generated/api-client\n"
  end

  test "an environment name longer than 36 characters is printed whole on a line of its own, and where it runs starts on the next line, in the column" do
    overview

    assert_stdout "  staging-for-the-accounting-team-in-leeds\n" \
                  "                                          staging.billing.example.com (Fly.io)\n" \
                  "  production                              billing.example.com (Fly.io)\n"
  end

  test "in NEEDS ATTENTION an id longer than 36 characters is printed whole on a line of its own, and a path too long for the line is printed whole, never broken, with the words after it on the next line" do
    start_repository
    edit_file "src/main/java/com/example/billing/reminders/OverdueInvoiceReminderController.java"

    overview

    assert_stdout "NEEDS ATTENTION\n" \
                  "  send-payment-reminder-to-overdue-clients-by-email\n" \
                  "                                         src/main/java/com/example/billing/reminders/OverdueInvoiceReminderController.java\n" \
                  "                                         changed since this record was updated\n"
  end
end
