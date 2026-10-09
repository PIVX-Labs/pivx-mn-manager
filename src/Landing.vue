<script setup lang="ts">
defineEmits<{ enter: [] }>();
</script>

<template>
  <div class="landing">
    <div class="glow glow-1" aria-hidden="true"></div>
    <div class="glow glow-2" aria-hidden="true"></div>

    <div class="landing-inner">
      <div class="shield" aria-hidden="true">
        <svg viewBox="0 0 96 96" width="84" height="84" role="img">
          <defs>
            <linearGradient id="shieldGrad" x1="0" y1="0" x2="1" y2="1">
              <stop offset="0" stop-color="#a06ee1" />
              <stop offset="1" stop-color="#5e4778" />
            </linearGradient>
          </defs>
          <path
            d="M48 6l34 13v27c0 21-14.5 35.5-34 42.5C28.5 81.5 14 67 14 46V19L48 6z"
            fill="url(#shieldGrad)"
          />
          <circle cx="48" cy="40" r="9.5" fill="#17122b" />
          <path d="M44.6 47h6.8l2 15H42.6l2-15z" fill="#17122b" />
          <path
            d="M48 30.5a9.5 9.5 0 019.5 9.5h-4a5.5 5.5 0 00-11 0h-4A9.5 9.5 0 0148 30.5z"
            fill="#d9c7f2"
            opacity="0.85"
          />
        </svg>
      </div>

      <span class="eyebrow">PIVX Labs</span>
      <h1>PIVX Masternode Manager</h1>
      <p class="tagline">
        Run PIVX masternodes without touching a terminal. Provision a VPS,
        watch collateral and status, cast governance votes — all from one app.
      </p>

      <div class="cta-row">
        <button class="cta-primary" type="button" @click="$emit('enter')">
          Manage Masternodes →
        </button>
        <a
          class="cta-ghost"
          href="https://github.com/PIVX-Labs/pivx-mn-manager"
          target="_blank"
          rel="noopener noreferrer"
        >
          View on GitHub
        </a>
      </div>

      <div class="features">
        <div class="feature-card">
          <div class="feature-icon">🛠️</div>
          <h3>One-click provisioning</h3>
          <p>
            Connect over SSH and the app installs a dockerized
            <code>pivxd</code> node for you — dependencies, firewall, the lot.
          </p>
        </div>
        <div class="feature-card">
          <div class="feature-icon">📡</div>
          <h3>Live status monitoring</h3>
          <p>
            Collateral and masternode state polled straight from the network,
            per node, with a single click.
          </p>
        </div>
        <div class="feature-card">
          <div class="feature-icon">🗳️</div>
          <h3>Governance voting</h3>
          <p>
            Sign and submit budget votes locally from your masternode key.
            Keys never leave the device.
          </p>
        </div>
      </div>

      <p class="footnote">
        Status monitoring works right here in the browser. Full VPS
        provisioning is available in the desktop build.
      </p>
    </div>
  </div>
</template>

<style scoped>
.landing {
  position: relative;
  min-height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
  background: radial-gradient(120% 120% at 20% 0%, #1d1535 0%, #0d0a1a 55%, #090713 100%);
  color: #f2eefc;
  font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  padding: 2.5rem 1.25rem;
}

.glow {
  position: absolute;
  border-radius: 50%;
  filter: blur(90px);
  opacity: 0.35;
  pointer-events: none;
}

.glow-1 {
  width: 420px;
  height: 420px;
  top: -140px;
  left: -100px;
  background: #5e4778;
}

.glow-2 {
  width: 360px;
  height: 360px;
  bottom: -120px;
  right: -80px;
  background: #862c6f;
}

.landing-inner {
  position: relative;
  max-width: 880px;
  text-align: center;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 1.1rem;
}

.shield {
  filter: drop-shadow(0 0 24px rgba(160, 110, 225, 0.45));
}

.eyebrow {
  font-size: 0.78rem;
  letter-spacing: 0.22em;
  text-transform: uppercase;
  color: #b79ae0;
  font-weight: 600;
}

h1 {
  margin: 0;
  font-size: clamp(2rem, 5vw, 3.2rem);
  line-height: 1.12;
  font-weight: 700;
  background: linear-gradient(100deg, #ffffff 35%, #d9c2fa 70%, #b98ef0 100%);
  -webkit-background-clip: text;
  background-clip: text;
  color: transparent;
}

.tagline {
  margin: 0;
  max-width: 560px;
  color: #c9c2dd;
  font-size: 1.05rem;
  line-height: 1.55;
}

.cta-row {
  display: flex;
  flex-wrap: wrap;
  gap: 0.8rem;
  justify-content: center;
  margin-top: 0.5rem;
}

.cta-primary {
  border: none;
  border-radius: 10px;
  padding: 0.85em 1.7em;
  font-size: 1rem;
  font-weight: 600;
  color: #ffffff;
  cursor: pointer;
  background: linear-gradient(100deg, #8b5cf6, #6d3fae);
  box-shadow: 0 6px 24px rgba(139, 92, 246, 0.35);
  transition: transform 0.15s ease, box-shadow 0.15s ease;
}

.cta-primary:hover {
  transform: translateY(-1px);
  box-shadow: 0 8px 30px rgba(139, 92, 246, 0.5);
}

.cta-primary:active {
  transform: translateY(0);
}

.cta-ghost {
  display: inline-flex;
  align-items: center;
  border: 1px solid rgba(255, 255, 255, 0.18);
  border-radius: 10px;
  padding: 0.85em 1.5em;
  font-size: 1rem;
  font-weight: 500;
  color: #e6def7;
  text-decoration: none;
  transition: border-color 0.15s ease, background-color 0.15s ease;
}

.cta-ghost:hover {
  border-color: rgba(160, 110, 225, 0.6);
  background: rgba(160, 110, 225, 0.08);
}

.features {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(230px, 1fr));
  gap: 1rem;
  width: 100%;
  margin-top: 1.6rem;
}

.feature-card {
  text-align: left;
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid rgba(255, 255, 255, 0.09);
  border-radius: 14px;
  padding: 1.2rem 1.3rem;
  transition: border-color 0.15s ease, transform 0.15s ease;
}

.feature-card:hover {
  border-color: rgba(160, 110, 225, 0.45);
  transform: translateY(-2px);
}

.feature-icon {
  font-size: 1.5rem;
  margin-bottom: 0.5rem;
}

.feature-card h3 {
  margin: 0 0 0.35rem;
  font-size: 1rem;
  font-weight: 600;
  color: #ffffff;
}

.feature-card p {
  margin: 0;
  font-size: 0.88rem;
  line-height: 1.5;
  color: #b3abc9;
}

.feature-card code {
  background: rgba(160, 110, 225, 0.15);
  border-radius: 4px;
  padding: 0.05em 0.35em;
  font-size: 0.85em;
}

.footnote {
  margin: 1.1rem 0 0;
  font-size: 0.82rem;
  color: #8b84a3;
}

@media (prefers-reduced-motion: reduce) {
  .cta-primary,
  .cta-ghost,
  .feature-card {
    transition: none;
  }
}
</style>
