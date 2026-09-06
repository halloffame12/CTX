package service

import (
	"fmt"
	"sync"

	"github.com/org/app/internal/db"
	"github.com/org/app/internal/models"
)

type Repository struct {
	mu     sync.Mutex
	session *db.Session
	items  map[string]*models.User
}

func (r *Repository) Find(pk string) (*models.User, error) {
	r.mu.Lock()
	defer r.mu.Unlock()
	return r.items[pk], nil
}

func (r *Repository) Save(u *models.User) error {
	r.mu.Lock()
	defer r.mu.Unlock()
	r.items[u.ID] = u
	return nil
}

type Greeter interface {
	Greet() string
}

func NewRepository(s *db.Session) *Repository {
	return &Repository{session: s, items: make(map[string]*models.User)}
}

var _ = func(x int) int {
	return x * 2
}
