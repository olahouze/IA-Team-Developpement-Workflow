import { openUrl } from '@tauri-apps/plugin-opener';
import { listen } from '@tauri-apps/api/event';

// Import healthchecks
import { isPgReady } from './healthcheck/pgTest';
import { isServerReady } from './healthcheck/serverTest';
import { isWorkerReady } from './healthcheck/workerTest';
import { isUrlReachable } from './healthcheck/urlTest';

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

      // Regex to strip ALL ANSI escape codes including formatting
      const cleanMessage = message.replace(/[\u001b\u009b][[()#;?]*(?:[0-9]{1,4}(?:;[0-9]{0,4})*)?[0-9A-ORZcf-nqry=><]/g, '').trim();

      span.textContent = `[${new Date().toLocaleTimeString()}] ${cleanMessage}`;
      container.appendChild(span);
      // Auto-scroll to bottom
      container.scrollTop = container.scrollHeight;
    }
  };

  const setProgress = (value: number, text: string) => {
    const pBar = document.getElementById('startup-progress') as HTMLProgressElement;
    const pLabel = document.getElementById('progress-label');
    if (pBar) pBar.value = value;
    if (pLabel) pLabel.textContent = text;
  };

  const setStatusReady = (elementId: string) => {
    const el = document.getElementById(elementId);
    if (el && !el.classList.contains('running')) {
      el.classList.add('running');
    }
  };

  // Setup periodic URL Reachability test
  let urlCheckInterval = setInterval(async () => {
    const isUp = await isUrlReachable('http://localhost:8000/');
    if (isUp) {
      setStatusReady('url-status');
      clearInterval(urlCheckInterval);
    }
  }, 2000);

  // Listen for logs from backend
  listen<string>('log-migration', (event) => {
    appendLog('migration-logs', event.payload);
    if (event.payload.includes('Restoring database')) {
      setProgress(40, 'Restauration de la base de données (Migration)...');
    }
    if (event.payload.includes('Migration restored successfully')) {
      setProgress(60, 'Migration terminée.');
    }
  });

  listen<string>('log-app', async (event) => {
    // Ignore spammy config logs
    if (event.payload.includes('Loaded ') && event.payload.includes(' setting to None')) {
      return;
    }

    const isError = event.payload.includes('[ERR]') || event.payload.includes('[CRITICAL]');
    appendLog('app-logs', event.payload, isError);

    const logText = event.payload;

    if (isPgReady(logText)) {
      setStatusReady('pg-status');
      setProgress(30, 'PostgreSQL démarré...');
    }

    if (isWorkerReady(logText)) {
      setStatusReady('worker-status');
      setProgress(75, 'Worker opérationnel...');
    }

    // Detect Windmill readiness
    if (isServerReady(logText)) {
      setStatusReady('server-status');
      setProgress(100, 'Serveur prêt ! Ouverture...');
      if (openBtn) {
        (openBtn as HTMLButtonElement).disabled = false;
        (openBtn as HTMLButtonElement).classList.remove('disabled');
      }

      // Auto-open
      try {
        console.log("Windmill is ready. Opening URL...");
        await openUrl('http://localhost:8000/');
        setProgress(100, 'Serveur Windmill en cours d\'exécution');
      } catch (err) {
        console.error("Failed to open URL:", err);
      }
    }
  });

  setProgress(10, 'Lancement des processus sidecars...');
});
