use sqlx::PgPool;

use crate::errors::AppError;
use crate::models::contact::{Contact, ContactFilter, CreateContact, UpdateContact};

pub async fn list(pool: &PgPool, filter: &ContactFilter) -> Result<Vec<Contact>, AppError> {
    let pag = crate::models::pagination::Pagination::from_filter(filter.page, filter.per_page);
    sqlx::query_as::<_, Contact>("SELECT * FROM contacts ORDER BY name LIMIT $1 OFFSET $2")
        .bind(pag.per_page)
        .bind(pag.offset)
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)
}

pub async fn count(pool: &PgPool) -> Result<i64, AppError> {
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM contacts")
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)?;
    Ok(row.0)
}

pub async fn create(pool: &PgPool, req: &CreateContact) -> Result<Contact, AppError> {
    sqlx::query_as::<_, Contact>(
        "INSERT INTO contacts (name, contact_type_id, farm_number, active, phone_cell,
         phone_home, phone_work, email, company_name, description)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10) RETURNING *",
    )
    .bind(&req.name)
    .bind(req.type_id)
    .bind(&req.farm_number)
    .bind(req.active)
    .bind(&req.phone_cell)
    .bind(&req.phone_home)
    .bind(&req.phone_work)
    .bind(&req.email)
    .bind(&req.company_name)
    .bind(&req.description)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn update(pool: &PgPool, id: i32, req: &UpdateContact) -> Result<Contact, AppError> {
    sqlx::query_as::<_, Contact>(
        "UPDATE contacts SET name = COALESCE($2, name),
         contact_type_id = COALESCE($3, contact_type_id),
         farm_number = COALESCE($4, farm_number),
         active = COALESCE($5, active),
         phone_cell = COALESCE($6, phone_cell),
         phone_home = COALESCE($7, phone_home),
         phone_work = COALESCE($8, phone_work),
         email = COALESCE($9, email),
         company_name = COALESCE($10, company_name),
         description = COALESCE($11, description)
         WHERE id = $1 RETURNING *",
    )
    .bind(id)
    .bind(&req.name)
    .bind(req.type_id)
    .bind(&req.farm_number)
    .bind(req.active)
    .bind(&req.phone_cell)
    .bind(&req.phone_home)
    .bind(&req.phone_work)
    .bind(&req.email)
    .bind(&req.company_name)
    .bind(&req.description)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn delete(pool: &PgPool, id: i32) -> Result<(), AppError> {
    let result = sqlx::query("DELETE FROM contacts WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await
        .map_err(AppError::Database)?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("Контакт {} не найден", id)));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::contact::{CreateContact, UpdateContact};

    fn create_req(name: &str) -> CreateContact {
        CreateContact {
            name: name.to_string(),
            type_id: None,
            farm_number: None,
            active: true,
            phone_cell: Some("+79991234567".into()),
            phone_home: None,
            phone_work: None,
            email: Some("test@test.com".into()),
            company_name: None,
            description: None,
        }
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_create_contact(pool: PgPool) {
        let contact = create(&pool, &create_req("Ivan")).await.unwrap();
        assert_eq!(contact.name, "Ivan");
        assert!(contact.active);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_list_contacts(pool: PgPool) {
        create(&pool, &create_req("A")).await.unwrap();
        create(&pool, &create_req("B")).await.unwrap();
        let filter = ContactFilter {
            page: None,
            per_page: None,
        };
        let contacts = list(&pool, &filter).await.unwrap();
        assert_eq!(contacts.len(), 2);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_update_contact(pool: PgPool) {
        let created = create(&pool, &create_req("Old")).await.unwrap();
        let req = UpdateContact {
            name: Some("New".into()),
            active: Some(false),
            type_id: None,
            farm_number: None,
            phone_cell: None,
            phone_home: None,
            phone_work: None,
            email: None,
            company_name: None,
            description: None,
        };
        let updated = update(&pool, created.id, &req).await.unwrap();
        assert_eq!(updated.name, "New");
        assert!(!updated.active);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_delete_contact(pool: PgPool) {
        let created = create(&pool, &create_req("Del")).await.unwrap();
        delete(&pool, created.id).await.unwrap();
        let filter = ContactFilter {
            page: None,
            per_page: None,
        };
        let contacts = list(&pool, &filter).await.unwrap();
        assert!(contacts.is_empty());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_delete_nonexistent(pool: PgPool) {
        let result = delete(&pool, 99999).await;
        assert!(result.is_err());
    }
}
