use axum::{
    routing::get,
    Router,
    http::{StatusCode, Request, header},
    response::{IntoResponse, Html},
    middleware::{self, Next},
    body::Body,
};
use tower_http::services::ServeDir;
use base64::{engine::general_purpose, Engine as _};
use std::fs;

#[tokio::main]
async fn main() {
    let dossier_uploads = r"C:\Users\cleme\OneDrive\Bureau\projet\cdhv_site\personnalisation-boite-poche-main\api\uploads";
    let login = "admin_entreprise";
    let mdp = "MdpUltraSecret123/!";

    let auth_valide = format!("{}:{}", login, mdp);
    let expected = format!("Basic {}", general_purpose::STANDARD.encode(auth_valide));

    // Création du routeur
    let app = Router::new()
        // Page d'accueil qui liste les images
        .route("/fichiers/", get(move || async move {
            generer_page_galerie(dossier_uploads)
        }))
        // Service qui permet d'afficher les images elles-mêmes
        .nest_service("/src", ServeDir::new(dossier_uploads))
        .layer(middleware::from_fn(move |req: Request<Body>, next: Next| {
            let expected_clone = expected.clone();
            let auth_header = req.headers().get(header::AUTHORIZATION).and_then(|h| h.to_str().ok()).map(String::from);

            async move {
                if let Some(header_val) = auth_header {
                    if header_val == expected_clone {
                        return next.run(req).await;
                    }
                }
                (StatusCode::UNAUTHORIZED, [(header::WWW_AUTHENTICATE, r#"Basic realm="Espace Photos""#)], "Acces refuse").into_response()
            }
        }));

    println!("🚀 Galerie lancée sur http://127.0.0.1:8080/fichiers/");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// --- LA FONCTION QUI CREE TA PAGE WEB ---
fn generer_page_galerie(chemin: &str) -> Html<String> {
    let mut fichiers = Vec::new();

    if let Ok(entries) = fs::read_dir(chemin) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                let metadata = entry.metadata().unwrap();
                let modif = metadata.modified().unwrap();
                let nom = entry.file_name().into_string().unwrap_or_default();
                
                // On ne prend que les images
                if nom.ends_with(".jpg") || nom.ends_with(".png") || nom.ends_with(".jpeg") {
                    fichiers.push((nom, modif));
                }
            }
        }
    }

    // Tri du plus récent au plus vieux
    fichiers.sort_by(|a, b| b.1.cmp(&a.1));

    // Construction du HTML avec un peu de style pour que ce soit joli
    let mut html = String::from(r#"
        <html>
        <head>
            <meta charset="UTF-8">
            <style>
                body { font-family: sans-serif; background: #f4f4f4; padding: 20px; }
                .grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(200px, 1fr)); gap: 20px; }
                .card { background: white; padding: 10px; border-radius: 8px; shadow: 0 2px 5px rgba(0,0,0,0.1); text-align: center; }
                img { max-width: 100%; border-radius: 4px; height: 150px; object-fit: cover; }
                h1 { color: #333; }
                .tag { font-size: 0.8em; padding: 3px 8px; border-radius: 10px; color: white; margin-bottom: 5px; display: inline-block; }
                .recto { background: #e67e22; } .verso { background: #3498db; } .image { background: #2ecc71; }
            </style>
        </head>
        <body>
            <h1>Galerie des Uploads (Plus récents en premier)</h1>
            <div class="grid">
    "#);

    for (nom, _) in fichiers {
        // Détection de la catégorie par le nom du fichier
        let categorie = if nom.contains("recto") { "recto" }
                        else if nom.contains("verso") { "verso" }
                        else { "image" };

        html.push_str(&format!(r#"
            <div class="card">
                <span class="tag {}">{}</span><br>
                <a href="/src/{}" target="_blank">
                    <img src="/src/{}">
                </a>
                <p style="font-size: 10px; overflow-wrap: break-word;">{}</p>
            </div>
        "#, categorie, categorie.to_uppercase(), nom, nom, nom));
    }

    html.push_str("</div></body></html>");
    Html(html)
}