package com.flashcard.temples

import android.os.Bundle
import android.widget.Button
import android.widget.EditText
import android.widget.ImageView
import android.widget.TextView
import androidx.appcompat.app.AppCompatActivity
import com.flashcard.temples.templecore.TempleQuizEngine

class MainActivity : AppCompatActivity() {
    private lateinit var quizEngine: TempleQuizEngine
    private lateinit var templeName: TextView
    private lateinit var userAnswer: EditText
    private lateinit var submitBtn: Button
    private lateinit var nextBtn: Button
    private lateinit var statsTextView: TextView

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_main)

        // Initialize Rust engine
        try {
            quizEngine = TempleQuizEngine()

            // Add sample temples (will be replaced with @MonkScholar data)
            quizEngine.addTemple(
                1u,
                "Wat Phra Kaew",
                13.3613,
                100.5055,
                "assets/temple_1.jpg",
                "Temple of the Emerald Buddha"
            )
        } catch (e: Exception) {
            showError("Engine initialization failed: ${e.message}")
        }

        // Initialize UI views
        initializeViews()

        // Load first quiz card
        loadNextQuiz()
    }

    private fun initializeViews() {
        templeName = findViewById(R.id.temple_name)
        userAnswer = findViewById(R.id.user_answer)
        submitBtn = findViewById(R.id.submit_btn)
        nextBtn = findViewById(R.id.next_btn)
        statsTextView = findViewById(R.id.stats_text)

        submitBtn.setOnClickListener { checkAnswer() }
        nextBtn.setOnClickListener { loadNextQuiz() }
    }

    private fun loadNextQuiz() {
        try {
            val card = quizEngine.getRandomQuizCard("image")
            templeName.text = "Guess: ${card.quizType.uppercase()}"
            // In full implementation, load image from card.imagePath
            userAnswer.text.clear()
            updateStats()
        } catch (e: Exception) {
            showError("Failed to load quiz: ${e.message}")
        }
    }

    private fun checkAnswer() {
        val answer = userAnswer.text.toString()
        if (answer.isEmpty()) {
            showError("Please enter an answer")
            return
        }

        try {
            // Get the current temple name (simplified for now)
            val result = quizEngine.checkAnswer(1u, answer, "Wat Phra Kaew")
            val feedback = if (result.isCorrect) {
                "✓ Correct! +${result.score} points"
            } else {
                "✗ Incorrect. Try again!"
            }
            templeName.text = feedback
        } catch (e: Exception) {
            showError("Error: ${e.message}")
        }
    }

    private fun updateStats() {
        try {
            val stats = quizEngine.getStats()
            statsTextView.text = stats
        } catch (e: Exception) {
            showError("Failed to get stats: ${e.message}")
        }
    }

    private fun showError(message: String) {
        statsTextView.text = "Error: $message"
    }
}
