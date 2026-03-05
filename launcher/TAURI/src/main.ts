import { openUrl } from '@tauri-apps/plugin-opener';
import { listen } from '@tauri-apps/api/event';

window.addEventListener("DOMContentLoaded", () => {
  const openBtn = document.getElementById('open-btn');
  const logsBtn = document.getElementById('logs-btn');

  if (openBtn) {
    openBtn.addEventListener('click', async () => {
      // Open the Windmill dashboard in the default browser
      try {
        console.log("Tentative d'ouverture de l'URL...");
        await openUrl('http://localhost:8000/');
        console.log("URL ouverte avec succès");
      } catch (err) {
        console.error("Erreur lors de l'ouverture de l'URL:", err);
        alert("Erreur lors de l'ouverture du lien : " + err);
      }
    });
  }

  if (logsBtn) {
    logsBtn.addEventListener('click', () => {
      const logSection = document.getElementById('log-section');
      if (logSection) {
        logSection.classList.toggle('hidden');
      }
    });
  }

  // Handle Log Tabs
  const tabBtns = document.querySelectorAll('.log-tab-btn');
  tabBtns.forEach(btn => {
    btn.addEventListener('click', () => {
      // Deactivate all tabs
      tabBtns.forEach(b => b.classList.remove('active'));
      document.querySelectorAll('.log-viewport').forEach(v => v.classList.add('hidden'));

      // Activate clicked tab
      btn.classList.add('active');
      const targetId = (btn as HTMLElement).dataset.tab;
      if (targetId) {
        document.getElementById(targetId)?.classList.remove('hidden');
      }
    });
  });

  const appendLog = (containerId: string, message: string, isError = false) => {
    const container = document.getElementById(containerId);
    if (container) {
      const span = document.createElement('div');
      span.className = `log-line${isError ? ' error' : ''}`;
      span.textContent = `[${new Date().toLocaleTimeString()}] ${message}`;
      container.appendChild(span);
      // Auto-scroll to bottom
      container.scrollTop = container.scrollHeight;
    }
  };

  // Listen for logs from backend
  listen<string>('log-migration', (event) => {
    appendLog('migration-logs', event.payload);
  });

  listen<string>('log-app', (event) => {
    const isError = event.payload.includes('[ERR]') || event.payload.includes('[CRITICAL]');
    appendLog('app-logs', event.payload, isError);
  });

  // Auto-lancement après 5 secondes (le temps que les services Windmill Démarent)
  setTimeout(async () => {
    try {
      await openUrl('http://localhost:8000/');
    } catch (e) {
      console.warn("L'auto-lancement a échoué:", e);
    }
  }, 5000);
});
