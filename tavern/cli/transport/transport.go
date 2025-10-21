package transport

import "realm.pub/tavern/internal/ent"


type Transport interface {
	New(client *ent.Client, server any, upstream string) error
	ForwardRequests() error
}
