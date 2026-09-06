const PI = 3.14159;
const TAU = 6.28318;

function add(a, b) {
  return a + b;
}

function subtract(a, b) {
  return a - b;
}

function multiply(a, b) {
  return a * b;
}

const square = (x) => x * x;

module.exports = { add, subtract, multiply, square, PI, TAU };