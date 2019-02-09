# Data Structure

Multiplay gRPC server use MongoDB.

## Schema Design

#### User Collection

```
db.user.collection =  {
  "_id": ObjectId(),
  "name": "user name",
  "x": 0, # x-coordinate
  "y" 0, # y-coordinate
}
```

#### Room Collection

```
db.room.collection = {
  "_id" : ObjectId(),
  "name": "room name"
}
```

#### Member Collection

```
db.member.collection = {
  "_id": ObjectId(), 
  "user_id": 1,
  "room_id": 1 
}
```
