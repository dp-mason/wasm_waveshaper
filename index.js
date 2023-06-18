// Import the WebAssembly module
import * as wasm from './pkg/wasm_waveshaper.js';

// Get a reference to the button element
const button = document.getElementById('ui-box');

// Add a click event listener to the button
button.addEventListener('click', () => {
  console.log("Hello from event handling function");

  // Call your Wasm function when the button is clicked
  wasm.play_sine_wave();
});