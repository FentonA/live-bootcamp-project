use auth_service::Application;

#[tokio::main]
async fn main() {
    let app = Application::build("0.0.0.0:3000")
        .await
        .expect("Could not build the app");

    app.run().await.expect("Could not run app");
}
