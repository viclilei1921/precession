export function encryptOutput(path: string) {
  return path.toLowerCase().endsWith('.enc') ? path : `${path}.enc`;
}

export function decryptOutput(path: string) {
  return path.toLowerCase().endsWith('.enc') ? path.slice(0, -4) : `${path}.dec`;
}
