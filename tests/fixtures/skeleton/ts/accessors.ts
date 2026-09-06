export class Bank {
  private _balance: number = 0;
  get balance(): number {
    return this._balance;
  }
  set balance(v: number) {
    if (v < 0) throw new Error("negative");
    this._balance = v;
  }
  static create(): Bank {
    return new Bank();
  }
}
export const obj = {
  name: "x",
  greet(msg: string): string {
    return `${this.name}:${msg}`;
  },
};
