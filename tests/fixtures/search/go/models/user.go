package models

type User struct {
	ID   int64
	Name string
}

type UserStatus int

const (
	StatusActive   UserStatus = 1
	StatusInactive UserStatus = 2
)

var UserRegistry = map[string]*User{}

func (u *User) DisplayName() string {
	return u.Name
}

func (u *User) SetName(name string) {
	u.Name = name
}

func Greet() string {
	return "hello"
}