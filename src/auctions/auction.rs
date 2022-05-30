enum AuctionState {
    ONGOING,
    FINISHED,
}

pub struct Auction {
    title: String,
    seller: NodeID,
    initial_price: f32,
    current_price: f32,
    starting_time: Date<Time>,
    time_remaining: Time,
    state: AuctionState,
    auction_id: String,
}