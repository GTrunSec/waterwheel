use anyhow::Result;
use sqlx::postgres::PgDatabaseError;
use sqlx::PgPool;
use tide::{Response, StatusCode};

mod job;
mod project;
pub mod types;
pub mod util;

const PG_INTEGRITY_ERROR: &str = "23";

#[derive(Clone)]
pub struct State {
    pool: PgPool,
}

pub async fn serve() -> Result<()> {
    let state = State {
        pool: crate::db::get_pool(),
    };

    let mut app = tide::with_state(state);
    app.with(tide::log::LogMiddleware::new());

    // project
    app.at("/api/projects")
        .get(project::get_by_name)
        .post(project::create)
        .put(project::update);
    app.at("/api/projects/:id")
        .get(project::get_by_id)
        .delete(project::delete);
    app.at("/api/projects/:id/jobs").get(project::list_jobs);

    // job
    app.at("/api/jobs")
        .get(job::get_by_name)
        .post(job::create)
        .put(job::create);
    app.at("/api/jobs/:id")
        .get(job::get_by_id)
        .delete(job::delete);

    app.at("/api/jobs/:id/tokens").get(job::get_tokens);
    app.at("/api/jobs/:id/tokens/:trigger_datetime").get(job::get_token_trigger_datetime);

    app.at("/api/jobs/:id/triggers").get(job::get_triggers);

    // web UI

    app.at("/static").serve_dir("ui/dist/")?;

    app.at("/").get(|_req| async {
        let body = tide::Body::from_file("ui/dist/index.html").await?;
        Ok(Response::builder(StatusCode::Ok).body(body).build())
    });

    let host =
        std::env::var("WATERWHEEL_SERVER_ADDR").unwrap_or_else(|_| "127.0.0.1:8080".to_owned());

    app.listen(host).await?;

    Ok(())
}

pub fn pg_error<T>(res: sqlx::Result<T>) -> Result<std::result::Result<T, Box<PgDatabaseError>>> {
    match res {
        Ok(t) => Ok(Ok(t)),
        Err(err) => match err {
            sqlx::Error::Database(db_err) => Ok(Err(db_err.downcast::<PgDatabaseError>())),
            err => Err(err.into()),
        },
    }
}
