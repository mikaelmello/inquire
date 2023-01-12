macro_rules! cancel_prompt {
    ($backend:expr, $prompt_message:expr) => {{
        $backend.frame_setup()?;
        $backend.render_canceled_prompt($prompt_message)?;
        $backend.frame_finish()?;
        return Err(InquireError::OperationCanceled);
    }};
}

macro_rules! interrupt_prompt {
    () => {
        return Err(InquireError::OperationInterrupted)
    };
}

macro_rules! finish_prompt_with_answer {
    ($backend:expr, $prompt_message:expr, $formatted_answer:expr, $answer: expr) => {{
        $backend.frame_setup()?;
        $backend.render_prompt_with_answer($prompt_message, $formatted_answer)?;
        $backend.frame_finish()?;

        return Ok($answer);
    }};
}
