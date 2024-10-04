import { Calendar } from "@fullcalendar/core";
import dayGridPlugin from "@fullcalendar/daygrid";
import timeGridPlugin from "@fullcalendar/timegrid";

console.log(window.calendar);

let calendarEl = document.getElementById("calendar");
let calendar = new Calendar(calendarEl, {
  plugins: [dayGridPlugin, timeGridPlugin],
  initialView: "timeGridWeek",
  headerToolbar: {
    left: "prev,next today",
    // center: "title",
    // right: "dayGridMonth,timeGridWeek,listWeek",
    right: "dayGridMonth,timeGridWeek",
    // left: "",
    center: "",
    // right: "",
  },
  weekends: false,
  height: "auto",
  events: window.calendar.map((event) => ({
    title: event.description,
    start: event.start,
    end: event.end,
    allDay: event.allDay,
    backgroundColor: event.backgroundColor,
  })),
  slotMinTime: "08:00:00",
  slotMaxTime: "19:00:00",
});
calendar.render();
