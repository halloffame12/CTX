export interface Account {
  id: string;
}

export class AccountService {
  accounts: Map<string, Account> = new Map();

  constructor(seed: Account[]) {
    seed.forEach((a) => this.accounts.set(a.id, a));
  }

  findByEmail(email: string): Account | undefined {
    for (const a of this.accounts.values()) {
      if (a.id === email) return a;
    }
    return undefined;
  }
}

export function accountById(id: string): Account | undefined {
  return undefined;
}