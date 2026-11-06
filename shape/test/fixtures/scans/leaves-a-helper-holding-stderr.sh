#!/bin/sh
# starts a helper that runs for 5 seconds and keeps the script's stderr open, as Spring's
# server does, then prints Atelier's 7 entrypoints and exits 0 at once.
sleep 5 >/dev/null &
cat <<'END_OF_OUTPUT'
# covers: http job mail
http	DELETE /bookings/:id	app/controllers/bookings_controller.rb	Bookings
http	GET /bookings/:id	app/controllers/bookings_controller.rb	Bookings
http	GET /workshops	app/controllers/workshops_controller.rb	Workshops
http	POST /bookings	app/controllers/bookings_controller.rb	Bookings
job	BookingReminderJob	app/jobs/booking_reminder_job.rb	Notifications
mail	BookingMailer#confirmation	app/mailers/booking_mailer.rb
mail	BookingMailer#reminder	app/mailers/booking_mailer.rb
END_OF_OUTPUT
