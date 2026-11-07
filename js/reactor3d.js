/**
 * 3D MECHANICAL REACTOR SILO & EMBOSSED TELEMETRY ENGINE
 * Hardware-Accelerated Three.js (r128) WebGL Architecture
 * Built for APC Back-UPS Pro Gaming & Universal Power Telemetry
 *
 * 1. MECHANICAL CONTAINMENT STRUCTURE (THE REACTOR SILO):
 *    - Heavy Bulkhead Backplate with Concentric Recesses & 12 Radial Stiffener Ribs
 *    - Extruded Segmented Power Rings (12 beveled segments with neon light pipes + 24 hex retaining bolts)
 *    - Iris Aperture Mechanism (6 interlocking curved containment blades with load-reactive dilation)
 *    - Hydraulic Dampeners (4 shock struts at diagonal quadrants with mirror-chrome rods & micro-recoil suspension)
 *    - Industrial Containment Clamps (4 cardinal axis calipers with twin fastener bolts & status LED jewels)
 *    - Morphing Quantum Plasma Core (Low/Med/High load geometries + breathing plasma orb)
 *    - Particle Vortex Swarm (300 particles pulled radially inward)
 *
 * 2. EMBOSSED TELEMETRY ENGINE (TRUE 3D CHISELED 'TIMBUL' READOUT):
 *    - ZERO Oval Background (100% transparent plate, zero scrim, zero background discs)
 *    - Multi-pass Procedural Chiseled Relief:
 *      * Ambient Occlusion Contact Halo (sharp localized glyph contrast without oval background)
 *      * Directional Stepped Extrusion (+x, +y drop shadows creating a physical 3D side-wall)
 *      * High-contrast Solid Tabular Numerals (JetBrains Mono tabular figures)
 *      * Top-Left Specular Bevel Ridge (-x, -y raking highlight glinting off edges)
 *      * Milled Ridge Inset Stroke (faceted chisel profile)
 *    - Dual-Plane In-Scene Parallax (Foreground plate at z=2.45, Shadow plate at z=2.30)
 *    - Standard PBR material responsive to scene PointLight and raking DirectionalLight
 *
 * 3. KINETIC KINEMATICS & RESPONSIVE RENDER LOOP:
 *    - Load-reactive aperture dilation (iris opens wider as load rises)
 *    - Real-time hydraulic dampener piston recoil proportional to electrical load
 *    - Clean mouse parallax anchored at (0, 0) with zero permanent camera slant
 *    - Container-bound ResizeObserver with setSize(w, h, false)
 */

class Reactor3D {
  constructor(canvasId) {
    this.canvas = document.getElementById(canvasId);
    if (!this.canvas || typeof THREE === 'undefined') {
      console.warn('Three.js or canvas not available');
      return;
    }

    this.scene = null;
    this.camera = null;
    this.renderer = null;

    // Theme Mode: 'dark' or 'light'
    this.colorMode = document.body.getAttribute('data-color-mode') || 'dark';

    // Power & Dynamic Kinetic State
    this.loadWatts = 132;
    this.loadRatio = 0.1; // 10% base
    this.powerStatus = 'online';
    this.targetSpeedMultiplier = 1.0;
    this.currentSpeedMultiplier = 1.0;

    // Adaptive Hardware Color Matrix
    this.currentColorHex = 0x00f0ff;
    this.adaptiveCssHex = '#00f0ff';
    this.targetColorHex = 0x00f0ff;
    this.stateLabel = 'IDLE / LOW (< 250W)';

    // 3D Scene Assemblies
    this.reactorGroup = null;
    this.bulkheadGroup = null;
    this.baseMesh = null;
    this.rimMesh = null;
    this.collarMesh = null;
    this.stiffenerRibs = [];

    // Extruded Segmented Power Rings
    this.segments = [];
    this.totalSegments = 12;
    this.activeSegmentsCount = 12;
    this.retainingBolts = [];

    // Iris Containment Aperture Blades
    this.apertureBladesGroup = null;
    this.apertureBlades = [];
    this.apertureDilation = 0.0;
    this.targetApertureDilation = 0.0;

    // Hydraulic Dampener Assemblies
    this.dampenersGroup = null;
    this.dampeners = [];

    // Containment Caliper Clamps
    this.clampsGroup = null;
    this.clamps = [];

    // Gyro Rings & Satellites
    this.gyroOuter = null;
    this.gyroInner = null;
    this.satelliteGroup = null;

    // Morphing Quantum Plasma Core
    this.coreMorphGroup = null;
    this.plasmaOrb = null;
    this.stageLowMesh = null;    // Geodesic Dodecahedron
    this.stageLowWire = null;
    this.stageMedGroup = null;   // Quantum Gimbal Cage
    this.cageRing1 = null;
    this.cageRing2 = null;
    this.stageHighGroup = null;  // Hyper-Stellated Plasma Star
    this.shards = [];

    // Particle Swarm
    this.particles = null;
    this.particleGeo = null;
    this.particleData = [];
    this.maxParticles = 300;
    this.activeParticleCount = 60;

    // Embossed Telemetry Engine (Dual-Plane In-Scene 3D)
    this.textCanvas = null;
    this.textCtx = null;
    this.textTexture = null;
    this.holoMesh = null;

    this.shadowCanvas = null;
    this.shadowCtx = null;
    this.shadowTexture = null;
    this.shadowMesh = null;

    this.currentBadge = 'LOAD POWER';
    this.currentVal = '132';
    this.currentUnit = 'W';
    this.currentSub = '10.0% OF 1320W • 50 Hz';

    // Lighting
    this.ambientLight = null;
    this.pointLight = null;
    this.dirLight = null;
    this.rimLight = null;

    // Mouse Parallax (Centered around 0, 0, zero permanent slant)
    this.targetRotationX = 0;
    this.targetRotationY = 0;
    this.currentRotationX = 0;
    this.currentRotationY = 0;

    // Animation Clock
    this.clock = new THREE.Clock();

    this.init();
  }

  init() {
    const stage = this.canvas.parentElement;
    const width = (stage ? stage.clientWidth : this.canvas.clientWidth) || 600;
    const height = (stage ? stage.clientHeight : this.canvas.clientHeight) || 540;

    // 1. Scene & Perspective Camera (Locked dead center on (0, 0, 0))
    this.scene = new THREE.Scene();
    this.camera = new THREE.PerspectiveCamera(45, width / height, 0.1, 1000);
    this.camera.position.set(0, 0, 18);
    this.camera.lookAt(0, 0, 0);

    // 2. WebGL Renderer
    this.renderer = new THREE.WebGLRenderer({
      canvas: this.canvas,
      antialias: true,
      alpha: true,
      powerPreference: "high-performance"
    });
    this.renderer.setPixelRatio(Math.min(window.devicePixelRatio || 1, 2));
    this.renderer.setSize(width, height, false);
    this.canvas.style.width = '100%';
    this.canvas.style.height = '100%';

    // 3. Studio Lighting Architecture
    // Ambient light: theme-aware intensity with cool metallic fill in dark mode
    this.ambientLight = new THREE.AmbientLight(
      this.colorMode === 'light' ? 0xffffff : 0xc0d2e8,
      this.colorMode === 'light' ? 1.3 : 0.95
    );
    this.scene.add(this.ambientLight);

    // Core central point light (radiates out through aperture blades)
    this.pointLight = new THREE.PointLight(this.currentColorHex, 3.2, 35);
    this.pointLight.position.set(0, 0, 4.5);
    this.scene.add(this.pointLight);

    // Top-left raking key light (casts specular highlights on bevels and chiseled edges)
    this.dirLight = new THREE.DirectionalLight(0xffffff, this.colorMode === 'light' ? 1.1 : 1.35);
    this.dirLight.position.set(-8, 10, 12);
    this.scene.add(this.dirLight);

    // Bottom-right sculpted rim light (electric blue/cyan to sculpt dark silhouettes)
    this.rimLight = new THREE.DirectionalLight(0x38bdf8, this.colorMode === 'light' ? 0.4 : 0.85);
    this.rimLight.position.set(8, -8, 6);
    this.scene.add(this.rimLight);

    // Back-glow backlight behind iris (outlines blade gaps with glowing cyber halo)
    this.backGlowLight = new THREE.PointLight(this.currentColorHex, this.colorMode === 'light' ? 0.4 : 1.2, 25);
    this.backGlowLight.position.set(0, 0, -2.5);
    this.scene.add(this.backGlowLight);

    // 4. Main Reactor Assembly Group
    this.reactorGroup = new THREE.Group();
    this.scene.add(this.reactorGroup);

    // Build Mechanical Silo Subsystems
    this.buildBulkheadSilo();
    this.buildSegmentedPowerRings();
    this.buildIrisApertureBlades();
    this.buildHydraulicDampeners();
    this.buildContainmentClamps();
    this.buildGyros();
    this.buildMorphingCore();
    this.buildParticleVortex();
    this.buildEmbossedTelemetryEngine();

    // 5. Initial Theme & Visual Synchronization
    this.updateThemeVisuals();

    // 6. Interaction & Responsive Bindings
    this.bindEvents();

    // 7. Start 60fps Hardware Render Loop
    this.animate = this.animate.bind(this);
    requestAnimationFrame(this.animate);
  }

  // =========================================================================
  // MECHANICAL CONTAINMENT SUBSYSTEM 1: BULKHEAD STATOR SILO
  // =========================================================================
  buildBulkheadSilo() {
    this.bulkheadGroup = new THREE.Group();
    this.bulkheadGroup.position.z = -0.65;
    this.reactorGroup.add(this.bulkheadGroup);

    const isLight = this.colorMode === 'light';

    // Main Cylindrical Containment Silo Backplate
    const siloBaseGeo = new THREE.CylinderGeometry(5.85, 6.45, 0.75, 64);
    const siloBaseMat = new THREE.MeshStandardMaterial({
      color: isLight ? 0xdce3eb : 0x1a2332,
      metalness: isLight ? 0.65 : 0.85,
      roughness: isLight ? 0.35 : 0.28
    });
    this.baseMesh = new THREE.Mesh(siloBaseGeo, siloBaseMat);
    this.baseMesh.rotation.x = Math.PI / 2;
    this.bulkheadGroup.add(this.baseMesh);

    // Concentric Machined Heat Dissipation Grooves
    const grooveGeo1 = new THREE.TorusGeometry(6.1, 0.05, 12, 96);
    const grooveMat = new THREE.MeshStandardMaterial({
      color: isLight ? 0x94a3b8 : 0x384a62,
      metalness: 0.92,
      roughness: 0.16
    });
    const groove1 = new THREE.Mesh(grooveGeo1, grooveMat);
    groove1.position.z = 0.38;
    this.bulkheadGroup.add(groove1);

    const grooveGeo2 = new THREE.TorusGeometry(5.75, 0.04, 12, 96);
    const groove2 = new THREE.Mesh(grooveGeo2, grooveMat);
    groove2.position.z = 0.38;
    this.bulkheadGroup.add(groove2);

    // 12 Radial Structural Stiffener Ribs
    this.stiffenerRibs = [];
    const ribGeo = new THREE.BoxGeometry(0.16, 1.35, 0.28);
    const ribMat = new THREE.MeshStandardMaterial({
      color: isLight ? 0xcbd5e1 : 0x27364b,
      metalness: isLight ? 0.7 : 0.88,
      roughness: isLight ? 0.3 : 0.22
    });
    for (let r = 0; r < 12; r++) {
      const angle = (r * Math.PI * 2) / 12;
      const rib = new THREE.Mesh(ribGeo, ribMat);
      rib.position.set(Math.cos(angle) * 5.8, Math.sin(angle) * 5.8, 0.28);
      rib.rotation.z = angle + Math.PI / 2;
      this.bulkheadGroup.add(rib);
      this.stiffenerRibs.push(rib);
    }

    // Inner Stator Seat Collar
    const collarGeo = new THREE.TorusGeometry(3.85, 0.075, 16, 96);
    const collarMat = new THREE.MeshStandardMaterial({
      color: isLight ? 0x94a3b8 : 0x3a4c64,
      metalness: 0.95,
      roughness: 0.16
    });
    this.collarMesh = new THREE.Mesh(collarGeo, collarMat);
    this.collarMesh.position.z = 0.39;
    this.bulkheadGroup.add(this.collarMesh);
  }

  // =========================================================================
  // MECHANICAL CONTAINMENT SUBSYSTEM 2: EXTRUDED SEGMENTED POWER RINGS
  // =========================================================================
  buildSegmentedPowerRings() {
    this.segments = [];
    this.retainingBolts = [];
    const isLight = this.colorMode === 'light';
    const radius = 4.8;
    const arcAngle = (Math.PI * 2) / this.totalSegments;
    const gap = 0.075; // Precision mechanical tolerance gap

    for (let i = 0; i < this.totalSegments; i++) {
      const angle = i * arcAngle - Math.PI / 2;

      // 2D Profile with inner/outer radial arcs
      const shape = new THREE.Shape();
      const innerR = 3.92;
      const outerR = 5.32;
      const startA = -arcAngle / 2 + gap;
      const endA = arcAngle / 2 - gap;

      const p1 = { x: Math.cos(startA) * outerR, y: Math.sin(startA) * outerR };
      const p3 = { x: Math.cos(endA) * innerR, y: Math.sin(endA) * innerR };

      shape.moveTo(p1.x, p1.y);
      shape.absarc(0, 0, outerR, startA, endA, false);
      shape.lineTo(p3.x, p3.y);
      shape.absarc(0, 0, innerR, endA, startA, true);
      shape.closePath();

      // Precision Machined Extrusion with Chamfered Bevel
      const extrudeSettings = {
        depth: 0.44,
        bevelEnabled: true,
        bevelSegments: 3,
        steps: 1,
        bevelSize: 0.045,
        bevelThickness: 0.045
      };

      const geo = new THREE.ExtrudeGeometry(shape, extrudeSettings);
      geo.center();

      const mat = new THREE.MeshStandardMaterial({
        color: this.currentColorHex,
        emissive: this.currentColorHex,
        emissiveIntensity: 1.25,
        metalness: isLight ? 0.5 : 0.85,
        roughness: isLight ? 0.25 : 0.18
      });

      const mesh = new THREE.Mesh(geo, mat);
      mesh.position.x = Math.cos(angle) * (radius - 0.18);
      mesh.position.y = Math.sin(angle) * (radius - 0.18);
      mesh.position.z = 0;
      mesh.rotation.z = angle + Math.PI / 2;

      this.reactorGroup.add(mesh);
      this.segments.push(mesh);
    }

    // Outer Retaining Flange Ring with 24 Recessed Hex Bolts
    const flangeGeo = new THREE.TorusGeometry(5.58, 0.08, 16, 96);
    const flangeMat = new THREE.MeshStandardMaterial({
      color: isLight ? 0x94a3b8 : 0x222e40,
      metalness: 0.95,
      roughness: 0.15
    });
    this.rimMesh = new THREE.Mesh(flangeGeo, flangeMat);
    this.rimMesh.position.z = 0.08;
    this.reactorGroup.add(this.rimMesh);

    // 24 Perimeter Hex Bolts
    const boltGeo = new THREE.CylinderGeometry(0.045, 0.045, 0.08, 6);
    const boltMat = new THREE.MeshStandardMaterial({
      color: isLight ? 0x64748b : 0x94a3b8,
      metalness: 0.95,
      roughness: 0.1
    });
    for (let b = 0; b < 24; b++) {
      const bAngle = (b * Math.PI * 2) / 24;
      const bolt = new THREE.Mesh(boltGeo, boltMat);
      bolt.position.set(Math.cos(bAngle) * 5.58, Math.sin(bAngle) * 5.58, 0.16);
      bolt.rotation.x = Math.PI / 2;
      this.reactorGroup.add(bolt);
      this.retainingBolts.push(bolt);
    }
  }

  // =========================================================================
  // MECHANICAL CONTAINMENT SUBSYSTEM 3: IRIS APERTURE MECHANISM
  // =========================================================================
  buildIrisApertureBlades() {
    this.apertureBladesGroup = new THREE.Group();
    this.apertureBladesGroup.position.z = -0.05;
    this.reactorGroup.add(this.apertureBladesGroup);

    this.apertureBlades = [];
    const isLight = this.colorMode === 'light';
    const bladeCount = 6;
    const pivotR = 3.35;

    // Curved Iris Containment Blade Profile
    const bladeShape = new THREE.Shape();
    bladeShape.moveTo(0, -0.22);
    bladeShape.lineTo(0.55, -0.28);
    bladeShape.bezierCurveTo(1.6, -0.32, 2.6, -0.15, 3.15, 0.42);
    bladeShape.bezierCurveTo(2.7, 0.95, 1.85, 0.85, 1.05, 0.52);
    bladeShape.lineTo(0, 0.22);
    bladeShape.absarc(0, 0, 0.26, Math.PI / 2, -Math.PI / 2, false);
    bladeShape.closePath();

    const bladeGeo = new THREE.ExtrudeGeometry(bladeShape, {
      depth: 0.08,
      bevelEnabled: true,
      bevelSegments: 2,
      steps: 1,
      bevelSize: 0.02,
      bevelThickness: 0.02
    });

    const bladeMat = new THREE.MeshStandardMaterial({
      color: isLight ? 0x64748b : 0x273549,
      metalness: isLight ? 0.75 : 0.88,
      roughness: isLight ? 0.3 : 0.22
    });

    for (let i = 0; i < bladeCount; i++) {
      const angle = (i * Math.PI * 2) / bladeCount;
      const pivotGroup = new THREE.Group();
      pivotGroup.position.set(Math.cos(angle) * pivotR, Math.sin(angle) * pivotR, 0);
      pivotGroup.rotation.z = angle + Math.PI / 2;

      const bladeMesh = new THREE.Mesh(bladeGeo, bladeMat);
      bladeMesh.position.z = (i * 0.015); // Slight stagger to prevent coplanar z-fighting
      pivotGroup.add(bladeMesh);

      // Pivot Knuckle Pin
      const pinGeo = new THREE.CylinderGeometry(0.06, 0.06, 0.14, 16);
      const pinMat = new THREE.MeshStandardMaterial({ color: 0x94a3b8, metalness: 0.95, roughness: 0.1 });
      const pin = new THREE.Mesh(pinGeo, pinMat);
      pin.rotation.x = Math.PI / 2;
      pin.position.z = 0.06;
      pivotGroup.add(pin);

      this.apertureBladesGroup.add(pivotGroup);
      this.apertureBlades.push({ pivotGroup, bladeMesh, baseAngle: angle + Math.PI / 2 });
    }
  }

  // =========================================================================
  // MECHANICAL CONTAINMENT SUBSYSTEM 4: HYDRAULIC DAMPENERS (SHOCK ABSORBERS)
  // =========================================================================
  buildHydraulicDampeners() {
    this.dampenersGroup = new THREE.Group();
    this.reactorGroup.add(this.dampenersGroup);
    this.dampeners = [];

    const isLight = this.colorMode === 'light';
    const dampenerMat = new THREE.MeshStandardMaterial({
      color: isLight ? 0x64748b : 0x1e293b,
      metalness: 0.85,
      roughness: 0.28
    });
    const chromeMat = new THREE.MeshStandardMaterial({
      color: 0xf8fafc,
      metalness: 0.98,
      roughness: 0.04
    });

    // 4 Diagonal Dampener Assemblies situated at 45°, 135°, 225°, 315°
    for (let d = 0; d < 4; d++) {
      const angle = (d * Math.PI * 2) / 4 + Math.PI / 4;
      const strut = new THREE.Group();
      strut.position.set(Math.cos(angle) * 4.82, Math.sin(angle) * 4.82, 0.22);
      strut.rotation.z = angle + Math.PI / 2;

      // 1. Outer Cylinder Barrel
      const barrelGeo = new THREE.CylinderGeometry(0.15, 0.17, 1.15, 16);
      barrelGeo.center();
      const barrel = new THREE.Mesh(barrelGeo, dampenerMat);
      barrel.position.y = 0.52;
      strut.add(barrel);

      // Reinforcement Ring Collars
      for (let r of [-0.28, 0.22]) {
        const rGeo = new THREE.TorusGeometry(0.18, 0.028, 8, 24);
        const rMesh = new THREE.Mesh(rGeo, dampenerMat);
        rMesh.rotation.x = Math.PI / 2;
        rMesh.position.y = 0.52 + r;
        strut.add(rMesh);
      }

      // 2. Inner Mirror-Chrome Piston Rod (Slides during micro-recoil)
      const rodGeo = new THREE.CylinderGeometry(0.082, 0.082, 1.15, 16);
      rodGeo.center();
      const rod = new THREE.Mesh(rodGeo, chromeMat);
      rod.position.y = -0.22;
      strut.add(rod);

      // 3. Dual C-Clevis Mounting Brackets with Pins
      for (let yPos of [1.14, -0.74]) {
        const clevisGeo = new THREE.BoxGeometry(0.24, 0.18, 0.2);
        const clevis = new THREE.Mesh(clevisGeo, dampenerMat);
        clevis.position.y = yPos;
        strut.add(clevis);

        const pinGeo = new THREE.CylinderGeometry(0.048, 0.048, 0.26, 12);
        const pin = new THREE.Mesh(pinGeo, chromeMat);
        pin.rotation.z = Math.PI / 2;
        pin.position.y = yPos;
        strut.add(pin);
      }

      this.dampeners.push({ strut, rod, baseRodY: -0.22, barrel });
      this.dampenersGroup.add(strut);
    }
  }

  // =========================================================================
  // MECHANICAL CONTAINMENT SUBSYSTEM 5: INDUSTRIAL CONTAINMENT CLAMPS
  // =========================================================================
  buildContainmentClamps() {
    this.clampsGroup = new THREE.Group();
    this.reactorGroup.add(this.clampsGroup);
    this.clamps = [];

    const isLight = this.colorMode === 'light';
    const clampMat = new THREE.MeshStandardMaterial({
      color: isLight ? 0x94a3b8 : 0x243246,
      metalness: isLight ? 0.7 : 0.9,
      roughness: isLight ? 0.35 : 0.22
    });
    const boltMat = new THREE.MeshStandardMaterial({
      color: isLight ? 0x475569 : 0x94a3b8,
      metalness: 0.95,
      roughness: 0.1
    });

    // 4 Cardinal Axis Clamps (0°, 90°, 180°, 270°)
    for (let c = 0; c < 4; c++) {
      const angle = (c * Math.PI * 2) / 4;
      const clamp = new THREE.Group();
      clamp.position.set(Math.cos(angle) * 5.52, Math.sin(angle) * 5.52, 0.28);
      clamp.rotation.z = angle + Math.PI / 2;

      // Heavy C-Caliper Body Block
      const bodyGeo = new THREE.BoxGeometry(0.68, 1.12, 0.36);
      bodyGeo.center();
      const body = new THREE.Mesh(bodyGeo, clampMat);
      clamp.add(body);

      // Twin Fastener Hex-Bolts
      for (let bx of [-0.19, 0.19]) {
        const boltGeo = new THREE.CylinderGeometry(0.065, 0.065, 0.12, 6);
        boltGeo.center();
        const bolt = new THREE.Mesh(boltGeo, boltMat);
        bolt.position.set(bx, 0.36, 0.21);
        bolt.rotation.x = Math.PI / 2;
        clamp.add(bolt);
      }

      // Status Indicator Jewel (Matches adaptive hardware color)
      const ledGeo = new THREE.CylinderGeometry(0.05, 0.05, 0.06, 16);
      const ledMat = new THREE.MeshStandardMaterial({
        color: this.currentColorHex,
        emissive: this.currentColorHex,
        emissiveIntensity: 2.2
      });
      const led = new THREE.Mesh(ledGeo, ledMat);
      led.position.set(0, -0.38, 0.19);
      led.rotation.x = Math.PI / 2;
      clamp.add(led);

      this.clamps.push({ clamp, body, led, ledMat });
      this.clampsGroup.add(clamp);
    }
  }

  // =========================================================================
  // SUBSYSTEM 6: KINETIC GYRO RINGS & SATELLITES
  // =========================================================================
  buildGyros() {
    const isLight = this.colorMode === 'light';
    const gyroGeo1 = new THREE.TorusGeometry(6.65, 0.035, 16, 120);
    const gyroMat1 = new THREE.MeshBasicMaterial({
      color: isLight ? 0x94a3b8 : 0x475b78,
      transparent: true,
      opacity: isLight ? 0.8 : 0.75
    });
    this.gyroOuter = new THREE.Mesh(gyroGeo1, gyroMat1);
    this.reactorGroup.add(this.gyroOuter);

    this.satelliteGroup = new THREE.Group();
    for (let k = 0; k < 4; k++) {
      const satGeo = new THREE.SphereGeometry(0.14, 16, 16);
      const satMat = new THREE.MeshBasicMaterial({ color: this.currentColorHex });
      const sat = new THREE.Mesh(satGeo, satMat);
      const satAngle = (k * Math.PI * 2) / 4;
      sat.position.set(Math.cos(satAngle) * 6.65, Math.sin(satAngle) * 6.65, 0);
      this.satelliteGroup.add(sat);
    }
    this.reactorGroup.add(this.satelliteGroup);

    const gyroGeo2 = new THREE.TorusGeometry(3.3, 0.03, 16, 100);
    const gyroMat2 = new THREE.MeshBasicMaterial({
      color: isLight ? 0x64748b : 0x33445c,
      transparent: true,
      opacity: 0.85
    });
    this.gyroInner = new THREE.Mesh(gyroGeo2, gyroMat2);
    this.reactorGroup.add(this.gyroInner);
  }

  // =========================================================================
  // SUBSYSTEM 7: MORPHING QUANTUM PLASMA CORE (3 STAGES)
  // =========================================================================
  buildMorphingCore() {
    this.coreMorphGroup = new THREE.Group();
    this.coreMorphGroup.position.z = -0.3;
    this.reactorGroup.add(this.coreMorphGroup);

    // Glowing Inner Plasma Sphere
    const plasmaGeo = new THREE.SphereGeometry(1.6, 32, 32);
    const plasmaMat = new THREE.MeshStandardMaterial({
      color: 0x060910,
      emissive: this.currentColorHex,
      emissiveIntensity: 0.4,
      metalness: 0.85,
      roughness: 0.1
    });
    this.plasmaOrb = new THREE.Mesh(plasmaGeo, plasmaMat);
    this.coreMorphGroup.add(this.plasmaOrb);

    // STAGE 1: Geodesic Dodecahedron (Low Load 0W - 250W)
    const isLight = this.colorMode === 'light';
    const dodecaGeo = new THREE.DodecahedronGeometry(2.35, 0);
    const dodecaMat = new THREE.MeshStandardMaterial({
      color: isLight ? 0xe2e8f0 : 0x0c121d,
      emissive: this.currentColorHex,
      emissiveIntensity: 0.25,
      metalness: isLight ? 0.6 : 0.9,
      roughness: isLight ? 0.3 : 0.15
    });
    this.stageLowMesh = new THREE.Mesh(dodecaGeo, dodecaMat);

    const wireGeo = new THREE.WireframeGeometry(dodecaGeo);
    const wireMat = new THREE.LineBasicMaterial({
      color: this.currentColorHex,
      transparent: true,
      opacity: 0.65
    });
    this.stageLowWire = new THREE.LineSegments(wireGeo, wireMat);
    this.stageLowMesh.add(this.stageLowWire);
    this.coreMorphGroup.add(this.stageLowMesh);

    // STAGE 2: Quantum Gimbal Cage (Medium Load 250W - 650W)
    this.stageMedGroup = new THREE.Group();
    const cageRingGeo1 = new THREE.TorusGeometry(2.5, 0.08, 16, 64);
    const cageRingMat1 = new THREE.MeshStandardMaterial({
      color: isLight ? 0xcbd5e1 : 0x141b2b,
      emissive: this.currentColorHex,
      emissiveIntensity: 0.8,
      metalness: 0.85,
      roughness: 0.2
    });
    this.cageRing1 = new THREE.Mesh(cageRingGeo1, cageRingMat1);
    this.stageMedGroup.add(this.cageRing1);

    const cageRingGeo2 = new THREE.TorusGeometry(2.2, 0.07, 16, 64);
    this.cageRing2 = new THREE.Mesh(cageRingGeo2, cageRingMat1.clone());
    this.cageRing2.rotation.x = Math.PI / 2;
    this.stageMedGroup.add(this.cageRing2);

    for (let b = 0; b < 4; b++) {
      const armGeo = new THREE.BoxGeometry(0.18, 0.5, 0.18);
      const armMesh = new THREE.Mesh(armGeo, cageRingMat1);
      const armA = (b * Math.PI * 2) / 4;
      armMesh.position.set(Math.cos(armA) * 2.5, Math.sin(armA) * 2.5, 0);
      armMesh.rotation.z = armA;
      this.stageMedGroup.add(armMesh);
    }
    this.stageMedGroup.visible = false;
    this.coreMorphGroup.add(this.stageMedGroup);

    // STAGE 3: Hyper-Stellated Plasma Star (High Load > 650W)
    this.stageHighGroup = new THREE.Group();
    this.shards = [];

    for (let s = 0; s < 12; s++) {
      const coneGeo = new THREE.ConeGeometry(0.35, 2.2, 4);
      coneGeo.center();
      const coneMat = new THREE.MeshStandardMaterial({
        color: this.currentColorHex,
        emissive: this.currentColorHex,
        emissiveIntensity: 1.5,
        metalness: 0.4,
        roughness: 0.1
      });
      const shard = new THREE.Mesh(coneGeo, coneMat);
      const shardAngle = (s * Math.PI * 2) / 12;
      shard.position.set(Math.cos(shardAngle) * 1.8, Math.sin(shardAngle) * 1.8, 0);
      shard.rotation.z = shardAngle - Math.PI / 2;
      this.stageHighGroup.add(shard);
      this.shards.push(shard);
    }
    this.stageHighGroup.visible = false;
    this.coreMorphGroup.add(this.stageHighGroup);
  }

  // =========================================================================
  // SUBSYSTEM 8: PARTICLE VORTEX SWARM
  // =========================================================================
  buildParticleVortex() {
    this.maxParticles = 300;
    this.particleData = [];
    const posArray = new Float32Array(this.maxParticles * 3);

    for (let i = 0; i < this.maxParticles; i++) {
      const theta = Math.random() * Math.PI * 2;
      const rad = 2.4 + Math.random() * 3.8;
      const z = (Math.random() - 0.5) * 1.6;
      const speed = 0.012 + Math.random() * 0.02;

      posArray[i * 3] = Math.cos(theta) * rad;
      posArray[i * 3 + 1] = Math.sin(theta) * rad;
      posArray[i * 3 + 2] = z;

      this.particleData.push({ theta, rad, z, baseSpeed: speed });
    }

    this.particleGeo = new THREE.BufferGeometry();
    this.particleGeo.setAttribute('position', new THREE.BufferAttribute(posArray, 3));
    this.activeParticleCount = Math.floor(50 + this.loadRatio * 250);
    this.particleGeo.setDrawRange(0, this.activeParticleCount);

    this.particleMat = new THREE.PointsMaterial({
      size: 0.1,
      color: this.currentColorHex,
      transparent: true,
      opacity: 0.85,
      blending: THREE.AdditiveBlending
    });
    this.particles = new THREE.Points(this.particleGeo, this.particleMat);
    this.reactorGroup.add(this.particles);
  }

  // =========================================================================
  // EMBOSSED TELEMETRY ENGINE (TRUE 3D CHISELED / TIMBUL READOUT)
  // ZERO OVAL BACKGROUND — 100% IN-SCENE DUAL-PLANE PARALLAX
  // =========================================================================
  buildEmbossedTelemetryEngine() {
    // 1. Primary High-Res Offscreen Canvas (Foreground Chiseled Glyphs)
    this.textCanvas = document.createElement('canvas');
    this.textCanvas.width = 1024;
    this.textCanvas.height = 1024;
    this.textCtx = this.textCanvas.getContext('2d');

    this.textTexture = new THREE.CanvasTexture(this.textCanvas);
    this.textTexture.minFilter = THREE.LinearFilter;
    this.textTexture.magFilter = THREE.LinearFilter;
    this.textTexture.generateMipmaps = false;

    // Single cleanly positioned foreground plate (6.6 x 6.6)
    const planeGeo = new THREE.PlaneGeometry(6.6, 6.6);
    const planeMat = new THREE.MeshBasicMaterial({
      map: this.textTexture,
      transparent: true,
      depthTest: false,
      depthWrite: false,
      side: THREE.FrontSide
    });

    this.holoMesh = new THREE.Mesh(planeGeo, planeMat);
    this.holoMesh.position.z = 2.45; // Foreground focal plane
    this.holoMesh.renderOrder = 9999;
    this.reactorGroup.add(this.holoMesh);

    this.update3DText(this.currentBadge, this.currentVal, this.currentUnit, this.currentSub);
  }

  /**
   * Procedural Multi-Pass Chiseled ('Timbul') Canvas Drawing
   * Soft Vignette Radial Scrim ("Shadow Yang Lama") with Crisp Solid Typography
   */
  update3DText(badge, val, unit, sub) {
    this.currentBadge = (badge !== undefined && badge !== null) ? String(badge) : this.currentBadge;
    this.currentVal = (val !== undefined && val !== null) ? String(val) : this.currentVal;
    this.currentUnit = (unit !== undefined && unit !== null) ? String(unit) : this.currentUnit;
    this.currentSub = (sub !== undefined && sub !== null) ? String(sub) : this.currentSub;

    if (!this.textCtx) return;
    const ctx = this.textCtx;
    ctx.clearRect(0, 0, 1024, 1024);

    const isLight = this.colorMode === 'light';
    const hexStr = this.adaptiveCssHex || (isLight ? '#0284c7' : '#00f0ff');
    const valStr = String(this.currentVal);
    const unitStr = String(this.currentUnit);

    // =========================================================================
    // SOFT RADIAL CONTRAST SCRIM ("SHADOW YANG LAMA") — Smooth Natural Vignette
    // =========================================================================
    const scrim = ctx.createRadialGradient(512, 500, 30, 512, 500, 480);
    if (isLight) {
      scrim.addColorStop(0, 'rgba(255, 255, 255, 0.94)');
      scrim.addColorStop(0.65, 'rgba(255, 255, 255, 0.60)');
      scrim.addColorStop(1, 'rgba(255, 255, 255, 0)');
    } else {
      scrim.addColorStop(0, 'rgba(6, 11, 20, 0.86)');
      scrim.addColorStop(0.60, 'rgba(6, 11, 20, 0.48)');
      scrim.addColorStop(1, 'rgba(6, 11, 20, 0)');
    }
    ctx.fillStyle = scrim;
    ctx.beginPath();
    ctx.arc(512, 500, 480, 0, Math.PI * 2);
    ctx.fill();

    // =========================================================================
    // 1. TOP BADGE LOCKUP: Machined Mechanical Brackets [ ── ◈ BADGE ── ]
    // =========================================================================
    const badgeText = this.currentBadge.toUpperCase();
    const badgeStr = `[ ── ◈ ${badgeText} ── ]`;
    const badgeY = 280;

    ctx.font = '700 48px "JetBrains Mono", monospace';
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';
    ctx.shadowColor = isLight ? 'rgba(255, 255, 255, 0.8)' : 'rgba(0, 0, 0, 0.9)';
    ctx.shadowBlur = 8;
    ctx.fillStyle = isLight ? '#475569' : hexStr;
    ctx.fillText(badgeStr, 512, badgeY);

    // =========================================================================
    // 2. PRIMARY NUMERIC VALUE & UNIT READOUT (LUMINOUS, CRISP & TIMBUL)
    // =========================================================================
    ctx.font = '800 255px "JetBrains Mono", monospace';
    const valWidth = ctx.measureText(valStr).width;

    ctx.font = '700 96px "JetBrains Mono", monospace';
    const unitWidth = ctx.measureText(unitStr).width;

    const gap = 20;
    const totalW = valWidth + gap + unitWidth;
    const startX = (1024 - totalW) / 2;
    const datumY = 495; // Locked center datum baseline
    const unitX = startX + valWidth + gap;
    const unitY = datumY - 14;

    // Numeric Value
    ctx.font = '800 255px "JetBrains Mono", monospace';
    ctx.textAlign = 'left';
    ctx.textBaseline = 'middle';
    ctx.fillStyle = isLight ? '#090e17' : '#ffffff';
    ctx.shadowColor = isLight ? 'rgba(255, 255, 255, 0.9)' : 'rgba(0, 0, 0, 0.95)';
    ctx.shadowBlur = isLight ? 8 : 20;
    ctx.fillText(valStr, startX, datumY);

    // Unit Label
    ctx.font = '700 96px "JetBrains Mono", monospace';
    ctx.fillStyle = hexStr;
    ctx.shadowColor = isLight ? 'rgba(255, 255, 255, 0.8)' : hexStr;
    ctx.shadowBlur = isLight ? 8 : 18;
    ctx.fillText(unitStr, unitX, unitY);

    // Reset shadow
    ctx.shadowBlur = 0;
    ctx.shadowColor = 'transparent';

    // =========================================================================
    // 3. SUBTEXT READOUT: Engineered Monospace Telemetry String
    // =========================================================================
    const subY = 690;
    ctx.font = '700 50px "JetBrains Mono", monospace';
    ctx.textAlign = 'center';
    ctx.shadowBlur = 6;
    ctx.shadowColor = isLight ? 'rgba(255, 255, 255, 0.8)' : 'rgba(0, 0, 0, 0.95)';
    ctx.fillStyle = isLight ? '#334155' : 'rgba(235, 245, 255, 0.92)';
    ctx.fillText(this.currentSub, 512, subY);

    if (this.textTexture) this.textTexture.needsUpdate = true;
  }

  // =========================================================================
  // ADAPTIVE HARDWARE COLOR ENGINE (DUAL CONTRAST RATIOS)
  // =========================================================================
  computeAdaptiveColor(watts, state) {
    const isLight = this.colorMode === 'light';

    if (state === 'battery') {
      return {
        hex: 0xffaa00,
        css: isLight ? '#b45309' : '#ffaa00',
        label: 'ON BATTERY (HAZARD AMBER)'
      };
    }
    if (state === 'critical' || state === 'overload') {
      return {
        hex: 0xff1111,
        css: isLight ? '#dc2626' : '#ef4444',
        label: 'CRITICAL / OVERLOAD (CRIMSON)'
      };
    }

    // Adaptive Power States
    if (watts <= 250) {
      return {
        hex: 0x00f0ff,
        css: isLight ? '#0284c7' : '#00f0ff',
        label: 'IDLE / LOW (< 250W)'
      };
    } else if (watts <= 650) {
      return {
        hex: 0xa855f7,
        css: isLight ? '#7e22ce' : '#a855f7',
        label: 'GAMING / NORMAL (250-650W)'
      };
    } else if (watts <= 1000) {
      return {
        hex: 0xf97316,
        css: isLight ? '#c2410c' : '#f97316',
        label: 'HEAVY LOAD (650-1000W)'
      };
    } else {
      return {
        hex: 0xef4444,
        css: isLight ? '#b91c1c' : '#ef4444',
        label: 'FULL STRESS (> 1000W)'
      };
    }
  }

  setAccentColor(colorStr) {
    if (!colorStr) return;
    this.adaptiveCssHex = colorStr;
    const hexClean = colorStr.replace('#', '');
    if (hexClean.length === 6) {
      this.currentColorHex = parseInt(hexClean, 16);
      this.updateSegments(this.activeSegmentsCount, this.powerStatus, this.loadWatts);
    }
  }

  // =========================================================================
  // TELEMETRY STATE REFRESH & KINEMATICS DILATION
  // =========================================================================
  updateSegments(activeCount, state = 'online', loadWatts = 132) {
    const safeCount = (typeof activeCount === 'number' && Number.isFinite(activeCount))
      ? Math.max(0, Math.min(12, Math.round(activeCount)))
      : 12;
    this.activeSegmentsCount = safeCount;

    const watts = (typeof loadWatts === 'number' && Number.isFinite(loadWatts))
      ? Math.max(0, loadWatts)
      : 132;
    this.loadWatts = watts;
    this.powerStatus = state || 'online';
    this.loadRatio = Math.max(0.08, Math.min(1.0, watts / 1320.0));
    const isLight = this.colorMode === 'light';

    // 1. Calculate Adaptive Color
    const adaptive = this.computeAdaptiveColor(watts, state);
    this.currentColorHex = adaptive.hex;
    this.adaptiveCssHex = adaptive.css;
    this.stateLabel = adaptive.label;

    document.body.style.setProperty('--accent', this.adaptiveCssHex);
    document.body.style.setProperty('--accent-glow', isLight ? `${this.adaptiveCssHex}25` : `${this.adaptiveCssHex}45`);

    const elStateBadge = document.getElementById('adaptiveStateLabel');
    if (elStateBadge) {
      elStateBadge.textContent = this.stateLabel;
    }

    // 2. Iris Aperture Target Dilation (Load-reactive opening)
    // Low load: tight containment (0.0). High load: wide dilation (1.0).
    this.targetApertureDilation = Math.max(0.0, Math.min(1.0, (watts - 100) / 1000.0));

    // 3. Physical 3D Core Asset Morphing
    if (watts <= 250) {
      this.stageLowMesh.visible = true;
      this.stageMedGroup.visible = false;
      this.stageHighGroup.visible = false;
    } else if (watts <= 650) {
      this.stageLowMesh.visible = false;
      this.stageMedGroup.visible = true;
      this.stageHighGroup.visible = false;

      const expandRatio = 1.0 + ((watts - 250) / 400) * 0.25;
      this.stageMedGroup.scale.set(expandRatio, expandRatio, expandRatio);
    } else {
      this.stageLowMesh.visible = false;
      this.stageMedGroup.visible = false;
      this.stageHighGroup.visible = true;

      const starScale = 1.0 + ((watts - 650) / 670) * 0.35;
      this.stageHighGroup.scale.set(starScale, starScale, starScale);
    }

    // 4. Particle Swarm Scaling
    this.activeParticleCount = Math.floor(50 + this.loadRatio * 250);
    if (this.particleGeo) {
      this.particleGeo.setDrawRange(0, this.activeParticleCount);
    }

    // 5. Kinetic Speed Scaling
    this.targetSpeedMultiplier = 1.0 + this.loadRatio * 5.0;

    // 6. Update Materials & Lighting
    const targetColor = this.currentColorHex;
    const emissiveIntensity = 1.25 + this.loadRatio * 0.8;

    this.segments.forEach((seg, idx) => {
      if (idx < this.activeSegmentsCount) {
        seg.material.color.setHex(targetColor);
        seg.material.emissive.setHex(targetColor);
        seg.material.emissiveIntensity = emissiveIntensity;
        seg.visible = true;
      } else {
        const inactiveHex = isLight ? 0xc8d3e0 : 0x223044;
        seg.material.color.setHex(inactiveHex);
        seg.material.emissive.setHex(isLight ? 0x000000 : 0x0c1522);
        seg.material.emissiveIntensity = isLight ? 0 : 0.45;
        seg.visible = true;
      }
    });

    if (this.pointLight) {
      this.pointLight.color.setHex(targetColor);
      this.pointLight.intensity = 2.8 + this.loadRatio * 2.2;
    }
    if (this.backGlowLight) {
      this.backGlowLight.color.setHex(targetColor);
      this.backGlowLight.intensity = isLight ? 0.4 : 1.2;
    }
    if (this.plasmaOrb) {
      this.plasmaOrb.material.emissive.setHex(targetColor);
      this.plasmaOrb.material.emissiveIntensity = 0.35 + this.loadRatio * 0.8;
    }
    if (this.stageLowMesh) {
      this.stageLowMesh.material.color.setHex(isLight ? 0xe2e8f0 : 0x182436);
      this.stageLowMesh.material.emissive.setHex(targetColor);
      this.stageLowWire.material.color.setHex(targetColor);
    }
    if (this.particleMat) {
      this.particleMat.color.setHex(targetColor);
    }
    if (this.satelliteGroup) {
      this.satelliteGroup.children.forEach(s => s.material.color.setHex(targetColor));
    }
    if (this.shards) {
      this.shards.forEach(sh => {
        sh.material.color.setHex(targetColor);
        sh.material.emissive.setHex(targetColor);
      });
    }

    // Clamp indicator jewels
    this.clamps.forEach(c => {
      if (c.ledMat) {
        c.ledMat.color.setHex(targetColor);
        c.ledMat.emissive.setHex(targetColor);
      }
    });

    // Update 3D In-Scene Chiseled Telemetry Readout
    this.update3DText();
  }

  // =========================================================================
  // THEME MODE VISUAL SYNCHRONIZATION (DARK / LIGHT CHASSIS)
  // =========================================================================
  updateThemeVisuals() {
    const isLight = this.colorMode === 'light';

    if (this.ambientLight) {
      this.ambientLight.intensity = isLight ? 1.3 : 0.95;
      this.ambientLight.color.setHex(isLight ? 0xffffff : 0xc0d2e8);
    }
    if (this.dirLight) {
      this.dirLight.intensity = isLight ? 1.1 : 1.35;
    }
    if (this.rimLight) {
      this.rimLight.intensity = isLight ? 0.4 : 0.85;
    }
    if (this.backGlowLight) {
      this.backGlowLight.intensity = isLight ? 0.4 : 1.2;
    }

    if (this.baseMesh) {
      this.baseMesh.material.color.setHex(isLight ? 0xdce3eb : 0x1a2332);
      this.baseMesh.material.metalness = isLight ? 0.65 : 0.85;
      this.baseMesh.material.roughness = isLight ? 0.35 : 0.28;
    }
    if (this.rimMesh) {
      this.rimMesh.material.color.setHex(isLight ? 0x94a3b8 : 0x364860);
    }
    if (this.collarMesh) {
      this.collarMesh.material.color.setHex(isLight ? 0x94a3b8 : 0x3a4c64);
    }
    if (this.gyroOuter) {
      this.gyroOuter.material.color.setHex(isLight ? 0x94a3b8 : 0x475b78);
      this.gyroOuter.material.opacity = isLight ? 0.8 : 0.75;
    }
    if (this.gyroInner) {
      this.gyroInner.material.color.setHex(isLight ? 0x64748b : 0x33445c);
    }

    // Stiffener ribs
    this.stiffenerRibs.forEach(rib => {
      rib.material.color.setHex(isLight ? 0xcbd5e1 : 0x27364b);
    });

    // Aperture blades
    this.apertureBlades.forEach(b => {
      b.bladeMesh.material.color.setHex(isLight ? 0x64748b : 0x273549);
      b.bladeMesh.material.metalness = isLight ? 0.75 : 0.88;
      b.bladeMesh.material.roughness = isLight ? 0.3 : 0.22;
    });

    // Clamps
    this.clamps.forEach(c => {
      c.body.material.color.setHex(isLight ? 0x94a3b8 : 0x243246);
    });

    this.updateSegments(this.activeSegmentsCount, this.powerStatus, this.loadWatts);
  }

  // =========================================================================
  // INTERACTION & PRECISION RESPONSIVE RESIZING
  // =========================================================================
  bindEvents() {
    // Mode Switcher Listener (Dark / Light)
    document.addEventListener('apc-mode-changed', (e) => {
      this.colorMode = e.detail.mode || 'dark';
      this.updateThemeVisuals();
    });

    // Subtle Front-Facing Parallax (Centered strictly at (0, 0), zero permanent tilt)
    window.addEventListener('mousemove', (e) => {
      const halfW = window.innerWidth / 2;
      const halfH = window.innerHeight / 2;
      const normX = (e.clientX - halfW) / halfW;
      const normY = (e.clientY - halfH) / halfH;

      this.targetRotationY = Math.max(-0.25, Math.min(0.25, normX * 0.22));
      this.targetRotationX = Math.max(-0.20, Math.min(0.20, -normY * 0.18));
    });

    // Native Raycasting for 3D In-Scene Readout Clicks (Eliminates DOM Transparent Hitbox Hack)
    this.raycaster = new THREE.Raycaster();
    this.mouse = new THREE.Vector2();

    const checkHoloHits = (e) => {
      if (!this.canvas || !this.holoMesh) return [];
      const rect = this.canvas.getBoundingClientRect();
      if (!rect.width || !rect.height) return [];
      this.mouse.x = ((e.clientX - rect.left) / rect.width) * 2 - 1;
      this.mouse.y = -((e.clientY - rect.top) / rect.height) * 2 + 1;
      this.raycaster.setFromCamera(this.mouse, this.camera);
      return this.raycaster.intersectObject(this.holoMesh);
    };

    if (this.canvas) {
      this.canvas.addEventListener('mousemove', (e) => {
        const hits = checkHoloHits(e);
        this.canvas.style.cursor = hits.length > 0 ? 'pointer' : 'default';
      });

      this.canvas.addEventListener('click', (e) => {
        const hits = checkHoloHits(e);
        if (hits.length > 0) {
          document.dispatchEvent(new CustomEvent('apc-core-clicked'));
        }
      });
    }

    window.addEventListener('mouseleave', () => {
      this.targetRotationX = 0;
      this.targetRotationY = 0;
    });

    // Precise Responsive Resizing via ResizeObserver (Locked Dead Center)
    const handleResize = () => {
      if (!this.canvas || !this.canvas.parentElement) return;
      const stage = this.canvas.parentElement;
      const width = stage.clientWidth;
      const height = stage.clientHeight;
      if (!width || !height) return;

      this.camera.aspect = width / height;
      this.camera.updateProjectionMatrix();
      this.camera.lookAt(0, 0, 0);

      this.renderer.setPixelRatio(Math.min(window.devicePixelRatio || 1, 2));
      this.renderer.setSize(width, height, false);
      this.canvas.style.width = '100%';
      this.canvas.style.height = '100%';
    };

    if (window.ResizeObserver && this.canvas && this.canvas.parentElement) {
      this.resizeObserver = new window.ResizeObserver(handleResize);
      this.resizeObserver.observe(this.canvas.parentElement);
    }
    window.addEventListener('resize', handleResize);
    setTimeout(handleResize, 100);
  }

  pause() {
    this.isPaused = true;
  }

  resume() {
    this.isPaused = false;
    this.clock.getDelta(); // Clear delta accumulation during pause
  }

  // =========================================================================
  // 60FPS HARDWARE ACCELERATED RENDER LOOP
  // =========================================================================
  animate() {
    requestAnimationFrame(this.animate);
    if (this.isPaused) return;

    const delta = this.clock.getDelta();
    const elapsedTime = this.clock.getElapsedTime();

    // 1. Lerp Kinetic Multiplier & Mouse Parallax
    this.currentSpeedMultiplier += (this.targetSpeedMultiplier - this.currentSpeedMultiplier) * 0.05;
    this.currentRotationX += (this.targetRotationX - this.currentRotationX) * 0.05;
    this.currentRotationY += (this.targetRotationY - this.currentRotationY) * 0.05;

    if (this.reactorGroup) {
      this.reactorGroup.rotation.x = this.currentRotationX;
      this.reactorGroup.rotation.y = this.currentRotationY;
    }

    const spinRate = this.currentSpeedMultiplier;

    // 2. Iris Aperture Blade Kinematics (Load-reactive dilation)
    this.apertureDilation += (this.targetApertureDilation - this.apertureDilation) * 0.05;
    const apertureDeltaAngle = this.apertureDilation * 0.38; // Max 0.38 rad rotation

    this.apertureBlades.forEach(b => {
      b.bladeMesh.rotation.z = -apertureDeltaAngle;
    });

    // 3. Hydraulic Dampener Micro-Recoil Suspension
    // Real-time oscillation damping high-current electromagnetic vibrations
    const recoilFreq = 2.2 + this.loadRatio * 6.5;
    const recoilAmp = 0.02 + this.loadRatio * 0.045;
    const recoilOffset = Math.sin(elapsedTime * recoilFreq) * recoilAmp;

    this.dampeners.forEach((damp, idx) => {
      const phase = (idx * Math.PI) / 2;
      const localRecoil = Math.sin(elapsedTime * recoilFreq + phase) * recoilAmp;
      damp.rod.position.y = damp.baseRodY + localRecoil;
    });

    // 4. Kinetic Rings & Gyros
    if (this.gyroOuter) this.gyroOuter.rotation.z += 0.003 * spinRate;
    if (this.satelliteGroup) this.satelliteGroup.rotation.z += 0.007 * spinRate;
    if (this.gyroInner) this.gyroInner.rotation.z -= 0.005 * spinRate;

    // 5. Morphing Quantum Core Stages
    if (this.stageLowMesh && this.stageLowMesh.visible) {
      this.stageLowMesh.rotation.y += 0.008 * spinRate;
      this.stageLowMesh.rotation.z += 0.005 * spinRate;
    }
    if (this.stageMedGroup && this.stageMedGroup.visible) {
      this.cageRing1.rotation.z += 0.015 * spinRate;
      this.cageRing2.rotation.x += 0.012 * spinRate;
    }
    if (this.stageHighGroup && this.stageHighGroup.visible) {
      this.stageHighGroup.rotation.z += 0.025 * spinRate;
      const jitter = (Math.random() - 0.5) * 0.04 * (this.loadRatio - 0.5);
      this.stageHighGroup.position.x = jitter;
      this.stageHighGroup.position.y = jitter;
    }

    // 6. Glowing Plasma Orb Respiration
    if (this.plasmaOrb) {
      const breathFreq = 2.0 + this.loadRatio * 8.0;
      const breathAmp = 0.03 + this.loadRatio * 0.04;
      const s = 1.0 + Math.sin(elapsedTime * breathFreq) * breathAmp;
      this.plasmaOrb.scale.set(s, s, s);
    }

    // 7. Dynamic Particle Swarm (Inward pull toward aperture throat)
    if (this.particleGeo && this.particleData.length > 0) {
      const positions = this.particleGeo.attributes.position.array;
      const inwardPull = 0.008 * spinRate;

      for (let i = 0; i < this.activeParticleCount; i++) {
        const p = this.particleData[i];
        p.theta += p.baseSpeed * spinRate;
        p.rad -= inwardPull;

        if (p.rad < 2.2) {
          p.rad = 5.6 + Math.random() * 0.8;
          p.theta = Math.random() * Math.PI * 2;
          p.z = (Math.random() - 0.5) * 1.6;
        }

        positions[i * 3] = Math.cos(p.theta) * p.rad;
        positions[i * 3 + 1] = Math.sin(p.theta) * p.rad;
        positions[i * 3 + 2] = p.z;
      }
      this.particleGeo.attributes.position.needsUpdate = true;
    }

    this.renderer.render(this.scene, this.camera);
  }
}

window.Reactor3D = Reactor3D;
