// How strong a password is, as advice: any password is accepted. A rough
// estimate of the guesses an attacker needs, from length, the kinds of
// characters, words, repeats and common passwords.

export type Level = 'veryWeak' | 'weak' | 'fair' | 'strong' | 'veryStrong'
export type Tip = 'common' | 'repeat' | 'short' | 'words' | 'good'

export interface Strength {
  level: Level
  /** 0–4, for a meter. */
  score: number
  /** Rough seconds to guess it offline, at the file's slow key derivation. */
  seconds: number
  tip: Tip
}

// The most common passwords, and words people tend to use for this app.
const COMMON = new Set([
  '123456', '123456789', '12345678', '12345', '1234567', '1234567890', '111111', '000000', '123123', '654321',
  'password', 'password1', 'passw0rd', 'qwerty', 'qwerty123', 'qwertyuiop', 'asdfgh', 'abc123', 'iloveyou', 'letmein',
  'welcome', 'admin', 'monkey', 'dragon', 'football', 'sunshine', 'princess', 'secret', 'master', 'login',
  'investor', 'investing', 'portfolio', 'money', 'finance', 'парола', 'парола123', 'пароль', 'passwort', 'motdepasse',
])

// Argon2id with the file's settings takes about a second on a laptop; a
// determined attacker with many fast machines might try ten thousand a second.
const GUESSES_PER_SECOND = 1e4

function charset(password: string): number {
  let size = 0
  if (/[a-z]/.test(password)) size += 26
  if (/[A-Z]/.test(password)) size += 26
  if (/[0-9]/.test(password)) size += 10
  if (/[^a-zA-Z0-9\p{L}]/u.test(password)) size += 33
  if (/[^\x00-\x7F]/.test(password) && /\p{L}/u.test(password)) size += 33
  return Math.max(size, 10)
}

/** Characters that add little: runs of the same character and simple sequences. */
function predictable(password: string): number {
  let count = 0
  for (let i = 2; i < password.length; i++) {
    const [a, b, c] = [password.charCodeAt(i - 2), password.charCodeAt(i - 1), password.charCodeAt(i)]
    if ((a === b && b === c) || (b - a === 1 && c - b === 1) || (a - b === 1 && b - c === 1)) count++
  }
  return count
}

export function strength(password: string): Strength {
  const lower = password.toLowerCase()
  if (COMMON.has(lower) || COMMON.has(lower.replace(/[\d!.?]+$/, ''))) {
    return { level: 'veryWeak', score: 0, seconds: 1, tip: 'common' }
  }
  const repeats = predictable(password)
  const effective = Math.max(1, [...password].length - repeats)
  let bits = effective * Math.log2(charset(password))
  // Several words joined by spaces or dashes: count each as one of many common words.
  const words = password.split(/[\s\-_.]+/).filter((w) => w.length >= 3)
  if (words.length >= 3) bits = Math.max(bits * 0.7, words.length * 11)
  else bits *= 0.75 // people pick predictable characters
  const seconds = Math.pow(2, bits) / GUESSES_PER_SECOND / 2
  const score = bits < 28 ? 0 : bits < 36 ? 1 : bits < 50 ? 2 : bits < 65 ? 3 : 4
  const level = (['veryWeak', 'weak', 'fair', 'strong', 'veryStrong'] as const)[score]
  const tip: Tip = repeats >= 2 ? 'repeat' : score >= 3 ? 'good' : words.length >= 2 ? 'short' : 'words'
  return { level, score, seconds, tip }
}

/** "about 3 hours", in the given language, or null beyond a thousand years. */
export function duration(seconds: number, lang: string): string | null {
  const units: [Intl.NumberFormatOptions['unit'], number][] = [
    ['year', 365 * 86400],
    ['month', 30 * 86400],
    ['day', 86400],
    ['hour', 3600],
    ['minute', 60],
    ['second', 1],
  ]
  if (seconds > 1000 * 365 * 86400) return null
  for (const [unit, size] of units) {
    if (seconds >= size || unit === 'second') {
      const n = Math.max(1, Math.round(seconds / size))
      return new Intl.NumberFormat(lang, { style: 'unit', unit, unitDisplay: 'long' }).format(n)
    }
  }
  return null
}
