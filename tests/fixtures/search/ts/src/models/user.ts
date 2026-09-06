export interface User {
  id: number;
  name: string;
}

export type UserId = string;

export const DEFAULT_ROLE = "user";

export enum UserRole {
  Admin,
  Member,
  Guest,
}

export class UserService {
  private users: User[] = [];

  public addUser(user: User): void {
    this.users.push(user);
  }

  getUser(id: UserId): User | undefined {
    return this.users.find((u) => u.id === Number(id));
  }
}