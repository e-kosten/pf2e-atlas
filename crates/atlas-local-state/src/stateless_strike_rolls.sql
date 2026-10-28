-- Display-only strikes have no downstream saved-state consumer. Keep initiative
-- and all event/preparation receipts; only remove identified strike payloads.
DELETE FROM encounter_roll_receipts
WHERE CASE WHEN json_valid(request_json) THEN json_extract(request_json, '$.action') END = 'strike'
  AND CASE WHEN json_valid(receipt_json) THEN json_extract(receipt_json, '$.payload.action') END = 'strike';
