/**
 * APC POWER CONSOLE — TAURI V2 FRONTEND IPC BRIDGE
 * Seamless zero-configuration bridge connecting web dashboard to Tauri v2 native core.
 * Intercepts `/api/*` REST fetch calls and translates them directly into Native Tauri IPC commands.
 */

(function () {
  'use strict';

  function isTauri() {
    return (
      typeof window !== 'undefined' &&
      (typeof window.__TAURI__ !== 'undefined' ||
        typeof window.__TAURI_INTERNALS__ !== 'undefined')
    );
  }

  async function invoke(cmd, args = {}) {
    if (
      window.__TAURI__ &&
      window.__TAURI__.core &&
      typeof window.__TAURI__.core.invoke === 'function'
    ) {
      return window.__TAURI__.core.invoke(cmd, args);
    }
    if (
      window.__TAURI_INTERNALS__ &&
      typeof window.__TAURI_INTERNALS__.invoke === 'function'
    ) {
      return window.__TAURI_INTERNALS__.invoke(cmd, args);
    }
    if (window.__TAURI__ && typeof window.__TAURI__.invoke === 'function') {
      return window.__TAURI__.invoke(cmd, args);
    }
    throw new Error(`Tauri invoke API not available for command: ${cmd}`);
  }

  function makeSyntheticResponse(data, status = 200, ok = true) {
    const jsonStr = typeof data === 'string' ? data : JSON.stringify(data);
    return {
      ok,
      status,
      statusText: ok ? 'OK' : 'Error',
      headers: new Headers({
        'Content-Type': 'application/json',
        'Cache-Control': 'no-cache',
      }),
      url: '',
      redirected: false,
      type: 'basic',
      bodyUsed: false,
      json: async () => data,
      text: async () => jsonStr,
      blob: async () => new Blob([jsonStr], { type: 'application/json' }),
      clone() {
        return makeSyntheticResponse(data, status, ok);
      },
    };
  }

  function parseBody(init) {
    if (!init || !init.body) return {};
    if (typeof init.body === 'string') {
      try {
        return JSON.parse(init.body);
      } catch (e) {
        return {};
      }
    }
    if (typeof init.body === 'object') return init.body;
    return {};
  }

  function initBridge() {
    if (!isTauri()) {
      return;
    }

    const originalFetch = window.fetch;

    window.fetch = async function (input, init = {}) {
      let urlStr =
        typeof input === 'string' ? input : input && input.url ? input.url : '';
      let pathname = urlStr;

      try {
        if (urlStr.startsWith('http://') || urlStr.startsWith('https://')) {
          pathname = new URL(urlStr).pathname;
        }
      } catch (e) {}

      const cleanPath = pathname.split('?')[0].split('#')[0];
      const method = (
        (init && init.method) ||
        (typeof input === 'object' && input.method) ||
        'GET'
      ).toUpperCase();

      // Route: GET /api/telemetry
      if (cleanPath === '/api/telemetry' && method === 'GET') {
        try {
          const telemetry = await invoke('get_telemetry');
          return makeSyntheticResponse(telemetry);
        } catch (err) {
          console.error('[TauriBridge] get_telemetry error:', err);
          return makeSyntheticResponse({ error: String(err) }, 500, false);
        }
      }

      // Route: GET /api/policy
      if (cleanPath === '/api/policy' && method === 'GET') {
        try {
          const policy = await invoke('get_policy');
          return makeSyntheticResponse(policy);
        } catch (err) {
          console.error('[TauriBridge] get_policy error:', err);
          return makeSyntheticResponse({ error: String(err) }, 500, false);
        }
      }

      // Route: POST /api/policy
      if (cleanPath === '/api/policy' && method === 'POST') {
        try {
          const body = parseBody(init);
          const savedPolicy = await invoke('save_policy', { newPolicy: body });
          return makeSyntheticResponse({
            success: true,
            policy: savedPolicy,
            ...savedPolicy,
          });
        } catch (err) {
          console.error('[TauriBridge] save_policy error:', err);
          return makeSyntheticResponse({ error: String(err) }, 400, false);
        }
      }

      // Route: POST /api/policy/reset
      if (cleanPath === '/api/policy/reset' && method === 'POST') {
        try {
          const resetPolicy = await invoke('reset_policy');
          return makeSyntheticResponse({
            success: true,
            policy: resetPolicy,
            ...resetPolicy,
          });
        } catch (err) {
          console.error('[TauriBridge] reset_policy error:', err);
          return makeSyntheticResponse({ error: String(err) }, 500, false);
        }
      }

      // Route: POST /api/abort_shutdown
      if (cleanPath === '/api/abort_shutdown' && method === 'POST') {
        try {
          const message = await invoke('abort_shutdown');
          return makeSyntheticResponse({ success: true, message });
        } catch (err) {
          console.error('[TauriBridge] abort_shutdown error:', err);
          return makeSyntheticResponse({ error: String(err) }, 500, false);
        }
      }

      // Route: POST /api/beeper
      if (cleanPath === '/api/beeper' && method === 'POST') {
        try {
          const body = parseBody(init);
          const enabled =
            typeof body.enabled === 'boolean' ? body.enabled : true;
          const beeperEnabled = await invoke('set_beeper', { enabled });
          return makeSyntheticResponse({
            success: true,
            beeper_enabled: beeperEnabled,
          });
        } catch (err) {
          console.error('[TauriBridge] set_beeper error:', err);
          return makeSyntheticResponse({ error: String(err) }, 400, false);
        }
      }

      // Route: GET /api/autostart
      if (cleanPath === '/api/autostart' && method === 'GET') {
        try {
          const enabled = await invoke('get_autostart');
          return makeSyntheticResponse({ enabled });
        } catch (err) {
          console.error('[TauriBridge] get_autostart error:', err);
          return makeSyntheticResponse({ enabled: false, error: String(err) }, 500, false);
        }
      }

      // Route: POST /api/autostart
      if (cleanPath === '/api/autostart' && method === 'POST') {
        try {
          const body = parseBody(init);
          const enabled = typeof body.enabled === 'boolean' ? body.enabled : true;
          const result = await invoke('set_autostart', { enabled });
          return makeSyntheticResponse({ success: true, enabled: result });
        } catch (err) {
          console.error('[TauriBridge] set_autostart error:', err);
          return makeSyntheticResponse({ success: false, error: String(err) }, 400, false);
        }
      }

      // Route: POST /api/selftest
      if (cleanPath === '/api/selftest' && method === 'POST') {
        try {
          const res = await invoke('run_selftest');
          return makeSyntheticResponse(res);
        } catch (err) {
          console.error('[TauriBridge] run_selftest error:', err);
          return makeSyntheticResponse({ success: false, error: String(err) }, 400, false);
        }
      }

      // Route: GET /api/events
      if (cleanPath === '/api/events' && method === 'GET') {
        try {
          let limit = null;
          let severity = null;
          let search = null;
          if (urlStr.includes('?')) {
            const params = new URLSearchParams(urlStr.split('?')[1]);
            if (params.has('limit')) limit = parseInt(params.get('limit'), 10);
            if (params.has('severity')) severity = params.get('severity');
            if (params.has('search')) search = params.get('search');
          }
          const events = await invoke('get_events', { limit, severity, search });
          return makeSyntheticResponse({ success: true, events });
        } catch (err) {
          console.error('[TauriBridge] get_events error:', err);
          return makeSyntheticResponse({ success: false, events: [], error: String(err) }, 500, false);
        }
      }

      // Route: POST /api/events/clear
      if (cleanPath === '/api/events/clear' && method === 'POST') {
        try {
          await invoke('clear_events');
          return makeSyntheticResponse({ success: true });
        } catch (err) {
          console.error('[TauriBridge] clear_events error:', err);
          return makeSyntheticResponse({ success: false, error: String(err) }, 500, false);
        }
      }

      // Route: GET /api/events/csv
      if (cleanPath === '/api/events/csv' && method === 'GET') {
        try {
          const csv = await invoke('export_events_csv');
          return makeSyntheticResponse({ success: true, csv });
        } catch (err) {
          console.error('[TauriBridge] export_events_csv error:', err);
          return makeSyntheticResponse({ success: false, error: String(err) }, 500, false);
        }
      }

      // Route: GET /api/energy
      if (cleanPath === '/api/energy' && method === 'GET') {
        try {
          const analytics = await invoke('get_energy_analytics');
          return makeSyntheticResponse({ success: true, ...analytics });
        } catch (err) {
          console.error('[TauriBridge] get_energy_analytics error:', err);
          return makeSyntheticResponse({ success: false, error: String(err) }, 500, false);
        }
      }

      // Route: POST /api/energy/reset_peaks
      if (cleanPath === '/api/energy/reset_peaks' && method === 'POST') {
        try {
          await invoke('reset_session_peaks');
          return makeSyntheticResponse({ success: true });
        } catch (err) {
          console.error('[TauriBridge] reset_session_peaks error:', err);
          return makeSyntheticResponse({ success: false, error: String(err) }, 500, false);
        }
      }

      // Passthrough for any other requests
      return originalFetch.apply(window, arguments);
    };

    window.__TAURI_BRIDGE__ = {
      isTauri: true,
      invoke,
    };
  }

  // Install immediately if in Tauri
  initBridge();
})();
