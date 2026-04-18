// @ts-check
const messageArea = /** @type {HTMLTextAreaElement | null} */ (
  document.getElementById("message-box")
);

const sendButton = /** @type {HTMLButtonElement | null} */ (
  document.getElementById("message-button")
);

const channelInput = /** @type {HTMLInputElement | null} */ (
  document.getElementById("channel-input")
);

const replyInput = /** @type {HTMLInputElement | null} */ (
  document.getElementById("reply-input")
);

const passwordInput = /** @type {HTMLInputElement | null} */ (
  document.getElementById("password-input")
);

sendButton?.addEventListener("click", async () => {
  if (!channelInput || !messageArea || !replyInput || !passwordInput) {
    console.warn("some input is missing!");
    return;
  }

  sendButton.ariaBusy = "true";

  const response = await fetch("send-message", {
    body: JSON.stringify({
      channelId: channelInput.value,
      message: messageArea.value,
      replyToId: replyInput.value || null,
    }),
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${passwordInput.value}`,
    },
    method: "POST",
  });

  const output = await response.json();

  alert(output.message);

  sendButton.ariaBusy = "false";
});
