CASE
    WHEN r.document_kind = 'JournalEntry' THEN 'journal_pack/' || r.pack_id
    WHEN r.document_kind = 'RollTable' THEN 'roll_table'
    WHEN r.document_kind = 'Actor' THEN CASE r.source_type
        WHEN 'npc' THEN 'creature'
        WHEN 'army' THEN 'army'
        WHEN 'character' THEN 'character'
        WHEN 'familiar' THEN 'familiar'
        WHEN 'hazard' THEN 'hazard'
        WHEN 'vehicle' THEN 'vehicle'
    END
    WHEN r.document_kind = 'Item' THEN CASE
        WHEN r.source_type IN ('action','affliction','ancestry','armor','background','class',
                               'condition','consumable','deity','effect','heritage','kit','shield',
                               'treasure','weapon') THEN r.source_type
        WHEN r.source_type = 'ammo' THEN 'ammunition'
        WHEN r.source_type = 'backpack' THEN 'container'
        WHEN r.source_type = 'equipment' THEN 'general_equipment'
        WHEN r.source_type = 'spell' THEN CASE
            WHEN (SELECT ritual FROM spell_projection WHERE root_record_id = r.record_id) = 1 THEN 'ritual'
            ELSE 'spell'
        END
        WHEN r.source_type = 'feat' THEN CASE
            (SELECT category FROM ability_projection WHERE root_record_id = r.record_id)
            WHEN 'ancestry' THEN 'ancestry_feat'
            WHEN 'ancestryfeature' THEN 'ancestry_feature'
            WHEN 'bonus' THEN 'bonus_feat'
            WHEN 'calling' THEN 'mythic_calling'
            WHEN 'class' THEN 'class_feat'
            WHEN 'classfeature' THEN 'class_feature'
            WHEN 'curse' THEN 'deity_curse'
            WHEN 'deityboon' THEN 'deity_boon'
            WHEN 'general' THEN 'general_feat'
            WHEN 'pfsboon' THEN 'society_boon'
            WHEN 'skill' THEN 'skill_feat'
        END
        WHEN r.source_type = 'campaignFeature' THEN CASE
            (SELECT category FROM ability_projection WHERE root_record_id = r.record_id)
            WHEN 'army-tactic' THEN 'army_tactic'
            WHEN 'army-war-action' THEN 'army_war_action'
            WHEN 'kingdom-activity' THEN 'kingdom_activity'
            WHEN 'kingdom-feat' THEN 'kingdom_feat'
            WHEN 'kingdom-feature' THEN 'kingdom_feature'
            ELSE 'kingdom_and_warfare'
        END
    END
END
