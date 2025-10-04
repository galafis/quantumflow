use crate::utils::types::{OrderBookLevel, OrderBookSnapshot, Ticker};
use anyhow::{Context, Result};
use chrono::Utc;
use futures::{SinkExt, StreamExt};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use tracing::{error, info};

#[derive(Debug, Deserialize)]
struct BinanceDepthUpdate {
    #[serde(rename = "e")]
    event_type: String,
    #[serde(rename = "s")]
    symbol: String,
    #[serde(rename = "b")]
    bids: Vec<[String; 2]>,
    #[serde(rename = "a")]
    asks: Vec<[String; 2]>,
}

#[derive(Debug, Deserialize)]
struct BinanceTickerUpdate {
    #[serde(rename = "e")]
    event_type: String,
    #[serde(rename = "s")]
    symbol: String,
    #[serde(rename = "b")]
    bid_price: String,
    #[serde(rename = "a")]
    ask_price: String,
    #[serde(rename = "c")]
    last_price: String,
    #[serde(rename = "v")]
    volume: String,
}

pub struct BinanceConnector {
    ws_url: String,
}

impl BinanceConnector {
    pub fn new() -> Self {
        Self {
            ws_url: "wss://stream.binance.com:9443/ws".to_string(),
        }
    }

    pub async fn connect_orderbook(
        &self,
        symbol: &str,
    ) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>> {
        let url = format!("{}{}@depth@100ms", self.ws_url, symbol.to_lowercase());
        info!("Connecting to Binance orderbook stream: {}", url);

        let (ws_stream, _) = connect_async(&url)
            .await
            .context("Failed to connect to Binance WebSocket")?;

        info!("Connected to Binance orderbook stream for {}", symbol);
        Ok(ws_stream)
    }

    pub async fn connect_ticker(
        &self,
        symbol: &str,
    ) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>> {
        let url = format!("{}{}@ticker", self.ws_url, symbol.to_lowercase());
        info!("Connecting to Binance ticker stream: {}", url);

        let (ws_stream, _) = connect_async(&url)
            .await
            .context("Failed to connect to Binance WebSocket")?;

        info!("Connected to Binance ticker stream for {}", symbol);
        Ok(ws_stream)
    }

    pub async fn stream_orderbook<F>(
        &self,
        symbol: &str,
        mut callback: F,
    ) -> Result<()>
    where
        F: FnMut(OrderBookSnapshot) + Send + 'static,
    {
        let mut ws_stream = self.connect_orderbook(symbol).await?;

        while let Some(msg) = ws_stream.next().await {
            match msg {
                Ok(tungstenite::Message::Text(text)) => {
                    match serde_json::from_str::<BinanceDepthUpdate>(&text) {
                        Ok(update) => {
                            let snapshot = self.convert_depth_update(update);
                            callback(snapshot);
                        }
                        Err(e) => {
                            error!("Failed to parse depth update: {}", e);
                        }
                    }
                }
                Ok(tungstenite::Message::Close(_)) => {
                    info!("WebSocket connection closed");
                    break;
                }
                Err(e) => {
                    error!("WebSocket error: {}", e);
                    break;
                }
                _ => {}
            }
        }

        Ok(())
    }

    pub async fn stream_ticker<F>(
        &self,
        symbol: &str,
        mut callback: F,
    ) -> Result<()>
    where
        F: FnMut(Ticker) + Send + 'static,
    {
        let mut ws_stream = self.connect_ticker(symbol).await?;

        while let Some(msg) = ws_stream.next().await {
            match msg {
                Ok(tungstenite::Message::Text(text)) => {
                    match serde_json::from_str::<BinanceTickerUpdate>(&text) {
                        Ok(update) => {
                            let ticker = self.convert_ticker_update(update);
                            callback(ticker);
                        }
                        Err(e) => {
                            error!("Failed to parse ticker update: {}", e);
                        }
                    }
                }
                Ok(tungstenite::Message::Close(_)) => {
                    info!("WebSocket connection closed");
                    break;
                }
                Err(e) => {
                    error!("WebSocket error: {}", e);
                    break;
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn convert_depth_update(&self, update: BinanceDepthUpdate) -> OrderBookSnapshot {
        let bids = update
            .bids
            .iter()
            .filter_map(|[price, qty]| {
                Some(OrderBookLevel {
                    price: Decimal::from_str(price).ok()?,
                    quantity: Decimal::from_str(qty).ok()?,
                })
            })
            .collect();

        let asks = update
            .asks
            .iter()
            .filter_map(|[price, qty]| {
                Some(OrderBookLevel {
                    price: Decimal::from_str(price).ok()?,
                    quantity: Decimal::from_str(qty).ok()?,
                })
            })
            .collect();

        OrderBookSnapshot {
            symbol: update.symbol,
            bids,
            asks,
            timestamp: Utc::now(),
        }
    }

    fn convert_ticker_update(&self, update: BinanceTickerUpdate) -> Ticker {
        Ticker {
            symbol: update.symbol,
            bid: Decimal::from_str(&update.bid_price).unwrap_or_default(),
            ask: Decimal::from_str(&update.ask_price).unwrap_or_default(),
            last: Decimal::from_str(&update.last_price).unwrap_or_default(),
            volume_24h: Decimal::from_str(&update.volume).unwrap_or_default(),
            timestamp: Utc::now(),
        }
    }
}

impl Default for BinanceConnector {
    fn default() -> Self {
        Self::new()
    }
}
