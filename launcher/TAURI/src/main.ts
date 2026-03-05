import { openUrl } from '@tauri-apps/plugin-opener';

window.addEventListener("DOMContentLoaded", () => {
  const openBtn = document.getElementById('open-btn');
  const logsBtn = document.getElementById('logs-btn');

  if (openBtn) {
    openBtn.addEventListener('click', async () => {
      // Open the Windmill dashboard in the default browser
      await openUrl('http://localhost:8000');
    });
  }

  if (logsBtn) {
    logsBtn.addEventListener('click', () => {
      alert("Viewing logs is not implemented yet in the UI, but you can see them in the terminal.");
    });
  }
});
