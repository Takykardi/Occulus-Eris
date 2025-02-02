use crate::bot::{
    commands::util::{get_settings, set_settings},
    tasks::{TaskInfo, TaskScheduler, TaskStorage},
    Error, Settings,
};
use serenity::prelude::{Context, RwLock, TypeMap};
use std::sync::Arc;

pub async fn add_task(
    ctx: &Context,
    handler: String,
    target_channel: u64,
    delay_secs: usize,
) -> Result<(), Error> {
    let tasks = { ctx.data.read().await.get::<TaskStorage>().unwrap().clone() };
    if tasks.is_valid_task(&handler) {
        let mut settings = get_settings(ctx).await?;
        let settings_mut = Arc::make_mut(&mut settings);

        for task in &settings_mut.active_tasks {
            if task.handler == handler {
                return Err("Task already exists.".into());
            }
        }

        log::info!("adding task {} to settings", handler);
        settings_mut.active_tasks.push(TaskInfo {
            handler: handler.clone(),
            target_channel,
            delay_secs,
        });

        set_settings(ctx, Arc::new(settings_mut.clone())).await?;

        if let Some(scheduler) = ctx.data.read().await.get::<TaskScheduler>() {
            scheduler.start_task(&handler).await;
        }
    } else {
        return Err("Task does not exist.".into());
    }

    Ok(())
}

pub async fn remove_task(ctx: &Context, handler: String) -> Result<(), Error> {
    let tasks = {
        ctx.data
            .read()
            .await
            .get::<TaskScheduler>()
            .unwrap()
            .clone()
    };

    if tasks.is_active(&handler).await {
        let mut settings = get_settings(ctx).await?;
        let settings_mut = Arc::make_mut(&mut settings);

        let mut index = None;
        for task in settings_mut.active_tasks.iter().enumerate() {
            if task.1.handler == handler {
                index = Some(task.0);
            }
        }

        if let Some(i) = index {
            let task = settings_mut.active_tasks.remove(i);
            tasks.stop_task(&task.handler).await;

            set_settings(ctx, Arc::new(settings_mut.clone())).await?;
        }
    }

    Ok(())
}
