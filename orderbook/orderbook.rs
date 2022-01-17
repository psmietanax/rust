use std::cmp::Reverse;
use rust_decimal::Decimal;
use std::collections::BTreeMap;
use linked_hash_map::LinkedHashMap;

#[derive(Debug)]
struct Orderbook {
    bids: BTreeMap<Reverse<Decimal>, LinkedHashMap<u64, Order>>,
    asks: BTreeMap<Decimal, LinkedHashMap<u64, Order>>
}

#[derive(Debug, Copy, Clone)]
struct Order {
    order_id: u64,
    side: Side,
    price: Decimal,
    quantity: u64,
}

#[derive(Debug, Copy, Clone)]
enum Side {
    Buy, Sell
}

impl Orderbook {
    fn new() -> Self {
        Orderbook { bids: BTreeMap::new(), asks: BTreeMap::new() }
    }

    fn get_orders(&self, side: Side) -> Vec<&Order> {
        match side {
            Side::Buy => {
                self.bids.values()
                    .flat_map(|list| list.values())
                    .collect()
            },
            Side::Sell => {
                self.asks.values()
                    .flat_map(|list| list.values())
                    .collect()
            }
        }
    }

    fn get_orders_at(&self, side: Side, price: Decimal) -> Vec<&Order> {
        match side {
            Side::Buy => {
                self.bids.get(&Reverse(price))
                    .map(|list| list.values().collect::<Vec<&Order>>())
                    .unwrap_or(vec![])
            },
            Side::Sell => {
                self.asks.get(&price)
                    .map(|list| list.values().collect::<Vec<&Order>>())
                    .unwrap_or(vec![])
            }
        }
    }

    fn get_best_price(&self, side: Side) -> Option<Decimal> {
        match side {
            Side::Buy => {
                self.bids.values()
                    .next()
                    .map(|list|
                        list.values()
                            .next()
                            .unwrap().price)
            },
            Side::Sell => {
                self.asks.values()
                    .next()
                    .map(|list|
                        list.values()
                            .next()
                            .unwrap().price)
            }
        }
    }

    fn add_order(&mut self, order: Order) {
        match order.side {
            Side::Buy => {
                self.bids.entry(Reverse(order.price))
                    .or_insert(LinkedHashMap::new())
                    .insert(order.order_id, order);

            },
            Side::Sell => {
                self.asks.entry(order.price)
                    .or_insert(LinkedHashMap::new())
                    .insert(order.order_id, order);
            }
        }
    }

    fn del_order(&mut self, order: Order) -> bool {
        match order.side {
            Side::Buy => {
                if let Some(list) = self.bids.get_mut(&Reverse(order.price)) {
                    return list.remove(&order.order_id).is_some();
                }
            },
            Side::Sell => {
                if let Some(list) = self.asks.get_mut(&order.price) {
                    return list.remove(&order.order_id).is_some();
                }
            }
        }
        false
    }

    fn get_total_quantity_at(&self, side: Side, price: Decimal) -> u64 {
        self.get_orders_at(side, price).iter()
            .fold(0, |x, &y| x + y.quantity)
    }

    fn get_total_volume_at(&self, side: Side, price: Decimal) -> Decimal {
        self.get_orders_at(side, price).iter()
            .fold(Decimal::ZERO, |x, &y| x + y.price * Decimal::from(y.quantity))
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_orders_empty() {
        let mut orderbook = Orderbook::new();
        assert_eq!(orderbook.get_orders(Side::Buy).is_empty(), true);
    }

    #[test]
    fn test_get_orders_buy() {
        let mut orderbook = Orderbook::new();
        orderbook.add_order(Order {
            order_id: 1,
            side: Side::Buy,
            price: Decimal::from(10),
            quantity: 100,
        });
        orderbook.add_order(Order {
            order_id: 2,
            side: Side::Buy,
            price: Decimal::from(30),
            quantity: 100,
        });
        orderbook.add_order(Order {
            order_id: 3,
            side: Side::Buy,
            price: Decimal::from(20),
            quantity: 100,
        });

        // prices in descending order
        let ids: Vec<u64> = orderbook.get_orders(Side::Buy)
            .iter()
            .map(|&order| order.order_id.clone())
            .collect();

        assert_eq!(ids, vec![2, 3, 1]);
    }

    #[test]
    fn test_get_orders_sell() {
        let mut orderbook = Orderbook::new();
        orderbook.add_order(Order {
            order_id: 1,
            side: Side::Sell,
            price: Decimal::from(10),
            quantity: 100,
        });
        orderbook.add_order(Order {
            order_id: 2,
            side: Side::Sell,
            price: Decimal::from(30),
            quantity: 100,
        });
        orderbook.add_order(Order {
            order_id: 3,
            side: Side::Sell,
            price: Decimal::from(20),
            quantity: 100,
        });

        // prices in ascending order
        let ids: Vec<u64> = orderbook.get_orders(Side::Sell)
            .iter()
            .map(|&order| order.order_id.clone())
            .collect();

        assert_eq!(ids, vec![1, 3, 2]);
    }

    #[test]
    fn test_add_orders_with_same_price() {
        let mut orderbook = Orderbook::new();
        orderbook.add_order(Order {
            order_id: 1,
            side: Side::Sell,
            price: Decimal::from(10),
            quantity: 10,
        });
        orderbook.add_order(Order {
            order_id: 2,
            side: Side::Sell,
            price: Decimal::from(10),
            quantity: 20,
        });
        orderbook.add_order(Order {
            order_id: 3,
            side: Side::Sell,
            price: Decimal::from(10),
            quantity: 30,
        });

        // prices in ascending order
        let ids: Vec<u64> = orderbook.get_orders(Side::Sell)
            .iter()
            .map(|&order| order.order_id.clone())
            .collect();

        assert_eq!(ids, vec![1, 2, 3]);
    }

    #[test]
    fn test_add_orders_at() {
        let mut orderbook = Orderbook::new();
        orderbook.add_order(Order {
            order_id: 1,
            side: Side::Sell,
            price: Decimal::from(20),
            quantity: 10,
        });
        orderbook.add_order(Order {
            order_id: 2,
            side: Side::Sell,
            price: Decimal::from(10),
            quantity: 20,
        });
        orderbook.add_order(Order {
            order_id: 3,
            side: Side::Sell,
            price: Decimal::from(10),
            quantity: 30,
        });

        // prices in ascending order
        let ids: Vec<u64> = orderbook.get_orders_at(Side::Sell, Decimal::from(10))
            .iter()
            .map(|&order| order.order_id.clone())
            .collect();

        assert_eq!(ids, vec![2, 3]);
    }

    #[test]
    fn test_get_best_price_buy() {
        let mut orderbook = Orderbook::new();
        orderbook.add_order(Order {
            order_id: 1,
            side: Side::Buy,
            price: Decimal::from(20),
            quantity: 100,
        });
        orderbook.add_order(Order {
            order_id: 2,
            side: Side::Buy,
            price: Decimal::from(30),
            quantity: 100,
        });

        assert_eq!(orderbook.get_best_price(Side::Buy), Some(Decimal::from(30)));
    }

    #[test]
    fn test_get_best_price_sell() {
        let mut orderbook = Orderbook::new();
        orderbook.add_order(Order {
            order_id: 1,
            side: Side::Buy,
            price: Decimal::from(10),
            quantity: 100,
        });

        assert_eq!(orderbook.get_best_price(Side::Sell), None);
    }

    #[test]
    fn test_get_total_quantity() {
        let mut orderbook = Orderbook::new();
        orderbook.add_order(Order {
            order_id: 1,
            side: Side::Sell,
            price: Decimal::from(10),
            quantity: 10,
        });
        orderbook.add_order(Order {
            order_id: 2,
            side: Side::Sell,
            price: Decimal::from(10),
            quantity: 20,
        });
        orderbook.add_order(Order {
            order_id: 3,
            side: Side::Sell,
            price: Decimal::from(20),
            quantity: 30,
        });

        assert_eq!(orderbook.get_total_quantity_at(Side::Sell, Decimal::from(10)), 30);
    }

    #[test]
    fn test_get_total_volume() {
        let mut orderbook = Orderbook::new();
        orderbook.add_order(Order {
            order_id: 1,
            side: Side::Sell,
            price: Decimal::from(10),
            quantity: 10,
        });
        orderbook.add_order(Order {
            order_id: 2,
            side: Side::Sell,
            price: Decimal::from(10),
            quantity: 20,
        });
        orderbook.add_order(Order {
            order_id: 3,
            side: Side::Sell,
            price: Decimal::from(20),
            quantity: 30,
        });

        assert_eq!(orderbook.get_total_volume_at(Side::Sell, Decimal::from(10)), Decimal::from(300));
    }

    #[test]
    fn test_del_order() {
        let mut orderbook = Orderbook::new();
        orderbook.add_order(Order {
            order_id: 1,
            side: Side::Buy,
            price: Decimal::from(10),
            quantity: 10,
        });

        let deleted = orderbook.del_order(Order {
            order_id: 1,
            side: Side::Buy,
            price: Decimal::from(10),
            quantity: 10,
        });

        assert_eq!(deleted, true);
        assert_eq!(orderbook.get_orders(Side::Buy).is_empty(), true);

        let deleted = orderbook.del_order(Order {
            order_id: 1,
            side: Side::Buy,
            price: Decimal::from(10),
            quantity: 10,
        });

        assert_eq!(deleted, false);
    }

}
