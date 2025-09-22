const buttons = document.querySelectorAll("button.delete-token");

for (const button of buttons) {
	button.addEventListener("click", async (event) => {
		const id = event.target.dataset.id;
		const response = await fetch(`/api/calendar/ics-token?token_id=${id}`, {
			method: "DELETE",
		});

		if (response.ok) {
			window.location.reload();
		}
	});
}

const createToken = document.querySelector("form");
const alert = document.querySelector(".alert");
const result = alert.querySelector("span");

createToken.addEventListener("submit", async (event) => {
	event.preventDefault();
	const formData = new FormData(event.target);
	const response = await fetch("/api/calendar/ics-token", {
		method: "POST",
		headers: {
			"Content-Type": "application/json",
		},
		body: JSON.stringify(Object.fromEntries(formData)),
	});

	if (!response.ok) {
		result.innerText = "An error occured while creating the token";
		alert.classList.add("alert-error");
		alert.classList.remove("alert-info");
		alert.classList.remove("hidden");
	}
	const token = await response.text();
	console.log(token);
	result.innerText = token;
	alert.classList.add("alert-info");
	alert.classList.remove("alert-error");
	alert.classList.remove("hidden");
});
