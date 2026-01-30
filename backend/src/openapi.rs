use utoipa::OpenApi;

use crate::routes;

#[derive(OpenApi)]
#[openapi(
    paths(
        routes::health::healthz,
        routes::admin::create_event,
        routes::admin::list_events,
        routes::admin::create_ticket_type,
        routes::admin::list_ticket_types,
        routes::seckill::seckill,
        routes::orders::get_order,
        routes::orders::pay_order,
    ),
    components(schemas(
        routes::health::HealthzResponse,
        routes::admin::CreateEventRequest,
        routes::admin::EventDto,
        routes::admin::CreateTicketTypeRequest,
        routes::admin::TicketTypeDto,
        routes::seckill::SeckillRequest,
        routes::seckill::OrderDto,
        routes::orders::OrderDto,
    )),
    tags(
        (name = "health", description = "Health check"),
        (name = "admin", description = "Admin endpoints (no auth in MVP)"),
        (name = "seckill", description = "Seckill / purchase"),
        (name = "orders", description = "Order read & simulated payment")
    )
)]
pub struct ApiDoc;
