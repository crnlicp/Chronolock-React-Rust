import '@testing-library/jest-dom/vitest';


if (typeof document !== 'undefined') {
  // Browser-like (jsdom) tests only.
  // Simulate the ic_env cookie the frontend canister sets.
  document.cookie = `ic_env=${encodeURIComponent(
    'PUBLIC_CANISTER_ID:backend=uxrrr-q7777-77774-qaaaq-cai&ic_root_key=308182301d060d2b0601040182dc7c0503010201060c2b0601040182dc7c05030201036100814c0e6ec71fab583b08bd81373c255c3c371b2e84863c98a4f1e08b74235d14fb5d9c0cd546d9685f913a0c0b2cc5341583bf4b4392e467db96d65b9bb4cb717112f8472e0d5a4d14505ffd7484b01291091c5f87b98883463f98091a0baaae',
  )}`;

  // Mock IndexedDB for DFINITY auth client
  class MockIDBRequest {
    result = {};
    error = null;
    onsuccess: ((event: any) => void) | null = null;
    onerror: ((event: any) => void) | null = null;
    onupgradeneeded: ((event: any) => void) | null = null;
    onblocked: ((event: any) => void) | null = null;
    readyState = 'done';
    source = null;
    transaction = null;

    constructor() {
      // Don't auto-trigger success to avoid unhandled errors in background
      // The auth client will handle this properly when needed
    }

    addEventListener(type: string, listener: (event: any) => void) {
      if (type === 'success') {
        this.onsuccess = listener;
      } else if (type === 'error') {
        this.onerror = listener;
      } else if (type === 'upgradeneeded') {
        this.onupgradeneeded = listener;
      } else if (type === 'blocked') {
        this.onblocked = listener;
      }
    }

    removeEventListener(_type: string, _listener: (event: any) => void) {
      // Mock implementation
    }
  }

  class MockIDBDatabase {
    createObjectStore() {
      return {};
    }

    transaction() {
      return {
        objectStore: () => ({
          get: () => new MockIDBRequest(),
          put: () => new MockIDBRequest(),
          delete: () => new MockIDBRequest(),
          add: () => new MockIDBRequest(),
        }),
      };
    }

    close() {}
  }

  // Define IDB classes globally
  Object.defineProperty(globalThis, 'IDBRequest', {
    value: MockIDBRequest,
    writable: true,
  });

  Object.defineProperty(globalThis, 'IDBDatabase', {
    value: MockIDBDatabase,
    writable: true,
  });

  Object.defineProperty(globalThis, 'IDBTransaction', {
    value: class MockIDBTransaction {
      objectStore() {
        return {
          get: () => new MockIDBRequest(),
          put: () => new MockIDBRequest(),
          delete: () => new MockIDBRequest(),
          add: () => new MockIDBRequest(),
        };
      }
    },
    writable: true,
  });

  Object.defineProperty(globalThis, 'IDBObjectStore', {
    value: class MockIDBObjectStore {
      get() {
        return new MockIDBRequest();
      }
      put() {
        return new MockIDBRequest();
      }
      delete() {
        return new MockIDBRequest();
      }
      add() {
        return new MockIDBRequest();
      }
    },
    writable: true,
  });

  Object.defineProperty(globalThis, 'IDBIndex', {
    value: class MockIDBIndex {
      get() {
        return new MockIDBRequest();
      }
      getKey() {
        return new MockIDBRequest();
      }
      getAll() {
        return new MockIDBRequest();
      }
      getAllKeys() {
        return new MockIDBRequest();
      }
      count() {
        return new MockIDBRequest();
      }
    },
    writable: true,
  });

  Object.defineProperty(globalThis, 'IDBCursor', {
    value: class MockIDBCursor {
      continue() {}
      advance() {}
      update() {
        return new MockIDBRequest();
      }
      delete() {
        return new MockIDBRequest();
      }
    },
    writable: true,
  });

  Object.defineProperty(globalThis, 'IDBKeyRange', {
    value: class MockIDBKeyRange {
      static bound() {
        return {};
      }
      static only() {
        return {};
      }
      static lowerBound() {
        return {};
      }
      static upperBound() {
        return {};
      }
    },
    writable: true,
  });

  Object.defineProperty(globalThis, 'indexedDB', {
    value: {
      open: () => {
        const request = new MockIDBRequest();
        request.result = new MockIDBDatabase();
        return request;
      },
      deleteDatabase: () => new MockIDBRequest(),
    },
    writable: true,
  });

  // Mock IntersectionObserver for react-intersection-observer
  Object.defineProperty(globalThis, 'IntersectionObserver', {
    value: class MockIntersectionObserver {
      root = null;
      rootMargin = '';
      thresholds: number[] = [];

      constructor(callback: IntersectionObserverCallback) {
        // Auto-trigger the callback to simulate element being in view
        setTimeout(() => {
          callback(
            [{ isIntersecting: true } as IntersectionObserverEntry],
            this as any,
          );
        }, 0);
      }
      observe() {}
      unobserve() {}
      disconnect() {}
      takeRecords(): IntersectionObserverEntry[] {
        return [];
      }
    },
    writable: true,
  });

  // Mock HTMLCanvasElement and WebGL for jsdom
  Object.defineProperty(HTMLCanvasElement.prototype, 'getContext', {
    value: (contextType: string) => {
      if (contextType === '2d') {
        return {
          fillRect: () => {},
          clearRect: () => {},
          getImageData: () => ({
            data: new Array(4).fill(0),
          }),
          putImageData: () => {},
          createImageData: () => ({ data: new Array(4).fill(0) }),
          setTransform: () => {},
          drawImage: () => {},
          save: () => {},
          fillText: () => {},
          restore: () => {},
          beginPath: () => {},
          moveTo: () => {},
          lineTo: () => {},
          closePath: () => {},
          stroke: () => {},
          translate: () => {},
          scale: () => {},
          rotate: () => {},
          arc: () => {},
          fill: () => {},
          measureText: () => ({ width: 0 }),
          transform: () => {},
          rect: () => {},
          clip: () => {},
        };
      } else if (
        contextType === 'webgl' ||
        contextType === 'experimental-webgl'
      ) {
        // Mock WebGL context
        return {
          createShader: () => ({}),
          pixelStorei: () => {},
          texSubImage2D: () => {},
          uniform2f: () => {},
          drawElements: () => {},
          shaderSource: () => {},
          compileShader: () => {},
          getShaderParameter: () => true,
          createProgram: () => ({}),
          attachShader: () => {},
          linkProgram: () => {},
          getProgramParameter: () => true,
          useProgram: () => {},
          createBuffer: () => ({}),
          bindBuffer: () => {},
          bufferData: () => {},
          createTexture: () => ({}),
          bindTexture: () => {},
          texImage2D: () => {},
          texParameteri: () => {},
          createFramebuffer: () => ({}),
          bindFramebuffer: () => {},
          framebufferTexture2D: () => {},
          getExtension: () => ({}),
          viewport: () => {},
          clear: () => {},
          enableVertexAttribArray: () => {},
          vertexAttribPointer: () => {},
          drawArrays: () => {},
          getUniformLocation: () => ({}),
          uniform1f: () => {},
          uniform2fv: () => {},
          uniform1i: () => {},
          activeTexture: () => {},
          enable: () => {},
          disable: () => {},
          blendFunc: () => {},
          getProgramInfoLog: () => '',
          getShaderInfoLog: () => '',
          checkFramebufferStatus: () => 36053, // FRAMEBUFFER_COMPLETE
          deleteTexture: () => {},
          deleteFramebuffer: () => {},
          deleteBuffer: () => {},
          deleteProgram: () => {},
          deleteShader: () => {},
          // Constants
          VERTEX_SHADER: 35633,
          FRAGMENT_SHADER: 35632,
          ARRAY_BUFFER: 34962,
          STATIC_DRAW: 35044,
          TEXTURE_2D: 3553,
          RGBA: 6408,
          UNSIGNED_BYTE: 5121,
          TEXTURE_WRAP_S: 10242,
          TEXTURE_WRAP_T: 10243,
          TEXTURE_MIN_FILTER: 10241,
          TEXTURE_MAG_FILTER: 10240,
          LINEAR: 9729,
          CLAMP_TO_EDGE: 33071,
          REPEAT: 10497,
          FRAMEBUFFER: 36160,
          COLOR_ATTACHMENT0: 36064,
          TEXTURE0: 33984,
          BLEND: 3042,
          SRC_ALPHA: 770,
          ONE_MINUS_SRC_ALPHA: 771,
          FLOAT: 5126,
          FRAMEBUFFER_COMPLETE: 36053,
        };
      }
      return null;
    },
  });
}
