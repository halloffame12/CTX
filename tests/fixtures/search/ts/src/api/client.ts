import { UserService } from "../models/user";
import { AccountService, accountById } from "../models/account";

export class ApiClient {
  private readonly userService: UserService;
  protected version = "2.0";

  constructor(userService: UserService, accountService: AccountService) {
    this.userService = userService;
    this.version = accountById(accountService.id)?.id ?? "2.0";
  }

  public async fetchUser(id: string) {
    return this.userService.getUser(id);
  }
}