# Example: Actor Chat Server

> A multi-actor system demonstrating concurrency patterns.

## actors/room.sp

```sploosh
actor ChatRoom {
    name: String,
    members: Vec<Handle<Client>>,

    fn init(name: String) -> Self {
        ChatRoom { name, members: Vec::new() }
    }

    pub fn join(&mut self, client: Handle<Client>) {
        self.members.push(client);
    }

    pub fn broadcast(&self, from: String, message: String) {
        let formatted = format("[{}] {}: {}", self.name, from, message);
        for member in self.members.iter() {
            send member.receive(formatted.clone());
        }
    }

    pub fn member_count(&self) -> u64 {
        self.members.len() as u64
    }
}

actor Client {
    username: String,
    room: Handle<ChatRoom>,

    fn init(username: String, room: Handle<ChatRoom>) -> Self {
        Client { username, room }
    }

    pub fn send_message(&self, message: String) {
        send self.room.broadcast(self.username.clone(), message);
    }

    pub fn receive(&mut self, message: String) {
        print(message);
    }
}
```

## main.sp

```sploosh
fn main() -> Result<(), AppError> {
    let room = spawn ChatRoom::init("general".into());

    let alice = spawn Client::init("Alice".into(), room.clone());
    let bob = spawn Client::init("Bob".into(), room.clone());

    send room.join(alice.clone());
    send room.join(bob.clone());

    send alice.send_message("Hello everyone!".into());
    send bob.send_message("Hey Alice!".into());

    let count = room.member_count();   // request/reply, blocks
    print(format("Room has {} members", count));

    Ok(())
}
```

## Key Patterns

- **`Handle<T>` is Clone + Send** -- freely passed between actors
- **`send` for fire-and-forget** -- non-blocking message dispatch
- **Direct call for request/reply** -- blocks until response
- **Owned parameters only** -- `String` not `&str` in pub methods
- **`.clone()` on handles and data** -- explicit copying for message passing
