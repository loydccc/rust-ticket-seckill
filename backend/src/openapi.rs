use utoipa::OpenApi;

use crate::routes;

#[derive(OpenApi)]
#[openapi(
    paths(
        routes::health::healthz,
        routes::auth::login,
        routes::admin::create_event,
        routes::admin::list_events,
        routes::admin::create_ticket_type,
        routes::admin::create_ticket_type_flat,
        routes::admin::list_ticket_types,
        routes::seckill::grab,
        routes::orders::my_orders,
        routes::orders::get_order,
        routes::orders::pay_order,
        routes::purchase_intents::create_intent,
        routes::purchase_intents::my_intents,
    ),
    components(schemas(
        routes::health::HealthzResponse,
        routes::auth::LoginRequest,
        routes::auth::LoginResponse,
        routes::admin::CreateEventRequest,
        routes::admin::EventDto,
        routes::admin::CreateTicketTypeRequest,
        routes::admin::CreateTicketTypeFlatRequest,
        routes::admin::TicketTypeDto,
        routes::seckill::GrabRequest,
        routes::seckill::OrderDto,
        routes::orders::OrderDto,
        routes::purchase_intents::CreateIntentRequest,
        routes::purchase_intents::IntentDto,
    )),
    tags(
        (name = "health", description = "Health check"),
        (name = "admin", description = "Admin endpoints (no auth in MVP)"),
        (name = "seckill", description = "Seckill / purchase"),
        (name = "orders", description = "Order read & simulated payment")
    )
)]
pub struct ApiDoc;
