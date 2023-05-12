<template>
  <div>
    <div id="textContainer">
      <div class="row">
        <div class="columnLeft">Roll:</div>
        <div class="columnRight">{{ roll }}°</div>
      </div>
      <div class="row">
        <div class="columnLeft">Pitch:</div>
        <div class="columnRight">{{ pitch }}°</div>
      </div>
      <div class="row">
        <div class="columnLeft">Yaw:</div>
        <div class="columnRight">{{ yaw }}°</div>
      </div>
    </div>
    <div id="container"></div>
  </div>
</template>

<script>

import * as Three from 'three';
import { OBJLoader } from 'three/addons/loaders/OBJLoader.js';

export default {
  name: 'OrientationVisualization',
  props: {
    orientation: {
      type: Array,
      required: true,
    },
  },
  methods: {
    init() {
        let container = document.getElementById('container');

        this.camera = new Three.PerspectiveCamera(35, container.clientWidth/container.clientHeight, 0.01, 100);
        this.camera.position.set(0, 10, 0);
        this.camera.lookAt(0, 0, 0);

        this.scene = new Three.Scene();
        const loader = new OBJLoader();

        loader.load(
          'rocket.obj',
          (object) => {
            this.mesh = object.children[0];
            this.mesh.material = new Three.MeshNormalMaterial();
            this.scene.add(this.mesh);
          },
          function(xhr) {
            console.log( ( xhr.loaded / xhr.total * 100 ) + '% loaded' );
          },
          function(error) {
            console.log( 'An error happened ' + error);
          }
        );

        this.renderer = new Three.WebGLRenderer({ antialias: true });
        this.renderer.setSize(container.clientWidth, container.clientHeight);
        this.renderer.setClearColor(0x000000, 1);

        this.renderer.setPixelRatio(window.devicePixelRatio);
        container.appendChild(this.renderer.domElement);

    },
    animate() {
        requestAnimationFrame(this.animate);
        this.renderer.clear();

        if (this.mesh == null || this.mesh == undefined) {
          return;
        }

        if (this.orientation != null && this.orientation != undefined) {
          const quaternion = new Three.Quaternion(
            this.orientation[0],
            this.orientation[1],
            this.orientation[2],
            this.orientation[3],
          );

          this.mesh.rotation.setFromQuaternion(quaternion);
        } else {
          this.mesh.rotation.set(Math.PI, 0, 0);
        }

        this.renderer.render(this.scene, this.camera);
    },
    orientationEuler() {
      if (this.orientation == null || this.orientation == undefined) {
        return new Three.Euler(0, 0, 0);
      }

      const quaternion = new Three.Quaternion(
        this.orientation[0],
        this.orientation[1],
        this.orientation[2],
        this.orientation[3],
      );
      const euler = new Three.Euler();
      euler.setFromQuaternion(quaternion);

      return euler;
    }
  },
  computed: {
    roll() {
      if (this.orientation == null || this.orientation == undefined) {
        return "?";
      }

      const quaternion = new Three.Quaternion(
        this.orientation[0],
        this.orientation[1],
        this.orientation[2],
        this.orientation[3],
      );
      const euler = new Three.Euler({ order: 'ZXY' });
      euler.setFromQuaternion(quaternion);

      const roll = euler.y;

      return (roll * (180.0 / Math.PI)).toFixed(1);
    },
    pitch() {
      return "N/A";
    },
    yaw() {
      return "N/A";
    },
  },
  mounted() {
      this.init();
      this.animate();
  }
}
</script>

<style scoped>

#container {
  width: 100%;
  height: 100%;
}

#textContainer {
  position: absolute;
	top: 10px;
	z-index: 100;
	display:block;
  color: white;
}

.row {
  display: flex;
  width: 7rem;
}

.columnLeft {
  flex: 50%;
  text-align: left;
}

.columnRight {
  flex: 50%;
  text-align: right;
}

</style>